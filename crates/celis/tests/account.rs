//! CIP-10, "The account".

mod support;

use celis_wire::{Account, Identity};
use support::{account, err, now, out, step_signed, Repo, PHRASE};

/// With no `$CELIS_HOME/phrase`, two runs at once mint one phrase, readable by its owner alone
/// (`spec/trails/10-decisions.md`, "The account").
#[test]
fn a_fresh_home_mints_one_phrase_for_its_owner_alone() {
    for _ in 0..30 {
        one_fresh_home_race();
    }
}

fn one_fresh_home_race() {
    use std::os::unix::fs::PermissionsExt;
    let repo = Repo::plain();
    let root = repo.path();
    let fresh = repo.home().with_file_name("fresh-home");
    let env = [("CELIS_HOME", fresh.to_str().unwrap())];
    let barrier = std::sync::Barrier::new(2);
    std::thread::scope(|scope| {
        let runs: Vec<_> = (0..2)
            .map(|_| {
                scope.spawn(|| {
                    barrier.wait();
                    repo.run(&root, &["decide", "--", "Show a tier."], &env, "")
                })
            })
            .collect();
        for run in runs {
            let o = run.join().unwrap();
            assert!(o.status.success(), "{o:?}");
        }
    });
    let phrase = fresh.join("phrase");
    assert_eq!(
        std::fs::metadata(&phrase).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let first = std::fs::read_to_string(&phrase).unwrap();
    assert!(Identity::from_phrase(first.trim()).is_ok(), "{first}");
    assert_eq!(first.split_whitespace().count(), 12);
    let o = repo.run(&root, &["decide", "--", "Show a tier."], &env, "");
    assert!(o.status.success(), "{o:?}");
    assert_eq!(std::fs::read_to_string(&phrase).unwrap(), first);
}

/// `read` shows a step the account signed, and one another account signed
/// (`spec/trails/10-decisions.md`, "The account").
#[test]
fn a_record_counts_whatever_account_signed_it() {
    let repo = Repo::plain();
    let at = now();
    let stranger = Account::generate().unwrap();
    let by_account = step_signed(&repo, &account(), "By the account.", at).id;
    let by_stranger = step_signed(&repo, &stranger, "By a stranger.", at).id;

    let o = repo.run(&repo.path(), &["past", "--json"], &[], "");
    assert!(o.status.success(), "{o:?}");
    let listed = out(&o);
    assert!(listed.contains(&by_account), "{listed}");
    assert!(listed.contains(&by_stranger), "{listed}");
}

/// Every command signs as the account, whatever session the agent runs in
/// (`spec/trails/10-decisions.md`, "The account").
#[test]
fn every_command_signs_as_the_account() {
    let repo = Repo::plain();
    let env = [("CELIS_SESSION", "s1"), ("CLAUDE_CODE_SESSION_ID", "cc-1")];
    let run = |args: &[&str]| {
        let o = repo.run(&repo.path(), args, &env, "");
        assert!(o.status.success(), "{args:?}: {o:?}");
        out(&o).trim().to_string()
    };
    let step = run(&["step", "add", "--", "work"]);
    run(&["step", "start", &step[..8]]);
    run(&["step", "done", &step[..8], "--as-planned", "--", "proof"]);
    run(&["decide", "--", "When you price: use cents."]);
    let files: Vec<_> = std::fs::read_dir(repo.store().join("shared"))
        .unwrap()
        .map(|e| std::fs::read_to_string(e.unwrap().path()).unwrap())
        .collect();
    assert_eq!(files.len(), 5);
    for text in files {
        let note: celis_wire::Note = serde_json::from_str(&text).unwrap();
        assert_eq!(note.pubkey, account().pubkey(), "{text}");
        assert!(note.tag("via").is_none(), "{text}");
    }
}

/// A phrase on stdin becomes the account every command signs as, the one the app gives for it, and the
/// key of older versions goes (`spec/trails/10-decisions.md`, "The account").
#[test]
fn auth_login_keeps_the_phrase_stdin_gives() {
    let repo = Repo::plain();
    std::fs::write(repo.home().join("key"), "0".repeat(64)).unwrap();
    let theirs = Identity::generate().unwrap();
    let o = repo.run(&repo.path(), &["auth", "login"], &[], &theirs.phrase());
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o).trim(), theirs.account().pubkey().npub());
    assert!(!repo.home().join("key").exists());
    let id = out(&repo.run(&repo.path(), &["step", "add", "--", "work"], &[], ""));
    let text = std::fs::read_dir(repo.store().join("shared"))
        .unwrap()
        .map(|e| std::fs::read_to_string(e.unwrap().path()).unwrap())
        .find(|t| t.contains(id.trim()))
        .unwrap();
    let note: celis_wire::Note = serde_json::from_str(&text).unwrap();
    assert_eq!(note.pubkey, theirs.account().pubkey());
}

/// `auth login` refuses a phrase given as words, and a text that is no phrase, and keeps its phrase
/// (`spec/trails/10-decisions.md`, "The account").
#[test]
fn auth_login_refuses_a_phrase_in_its_words_and_a_bad_phrase() {
    let repo = Repo::plain();
    let theirs = Identity::generate().unwrap().phrase();
    let mut args = vec!["auth", "login"];
    args.extend(theirs.split(' '));
    let o = repo.run(&repo.path(), &args, &[], "");
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    let o = repo.run(&repo.path(), &["auth", "login", "--", &theirs], &[], "");
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    let o = repo.run(&repo.path(), &["auth", "login"], &[], "not a phrase at all");
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    let kept = std::fs::read_to_string(repo.home().join("phrase")).unwrap();
    assert_eq!(kept.trim(), PHRASE);
}

/// `auth` has only `login`, `status` and `phrase`: `new` and `logout` are no commands, since a written
/// record mints a phrase and removing the file forgets it (`spec/trails/10-decisions.md`, "The account").
#[test]
fn auth_has_no_new_and_no_logout() {
    let repo = Repo::plain();
    for verb in ["new", "logout"] {
        let o = repo.run(&repo.path(), &["auth", verb], &[], "");
        assert_eq!(o.status.code(), Some(2), "{verb}: {}", err(&o));
    }
    assert_eq!(
        std::fs::read_to_string(repo.home().join("phrase"))
            .unwrap()
            .trim(),
        PHRASE
    );
}

/// `auth status` prints the npub and the path, never the phrase; `auth phrase` prints the phrase
/// (`spec/trails/10-decisions.md`, "The account").
#[test]
fn auth_status_shows_the_npub_and_auth_phrase_the_phrase() {
    let repo = Repo::plain();
    let o = repo.run(&repo.path(), &["auth", "status", "--json"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let shown: serde_json::Value = serde_json::from_str(out(&o).trim()).unwrap();
    assert_eq!(shown["npub"], account().pubkey().npub());
    let path = repo.home().join("phrase");
    assert_eq!(shown["path"], path.to_str().unwrap());
    assert!(!out(&o).contains("abandon"), "{}", out(&o));

    let o = repo.run(&repo.path(), &["auth", "phrase"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o).trim(), PHRASE);
}

/// With no phrase, `auth status` and `auth phrase` refuse and mint none
/// (`spec/trails/10-decisions.md`, "The account").
#[test]
fn auth_status_and_phrase_refuse_without_a_phrase() {
    let repo = Repo::plain();
    std::fs::remove_file(repo.home().join("phrase")).unwrap();
    for args in [&["auth", "status"][..], &["auth", "phrase"]] {
        let o = repo.run(&repo.path(), args, &[], "");
        assert_eq!(o.status.code(), Some(1), "{args:?}: {}", err(&o));
        assert!(err(&o).contains("celis auth login"), "{}", err(&o));
    }
    assert!(!repo.home().join("phrase").exists());
}

/// A command that writes no record mints no phrase, even where the store exists
/// (`spec/trails/10-decisions.md`, "The account").
#[test]
fn a_command_that_only_reads_mints_no_phrase() {
    let repo = Repo::plain();
    let o = repo.run(&repo.path(), &["step", "add", "--", "work"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    std::fs::remove_file(repo.home().join("phrase")).unwrap();
    for args in [&["past"][..], &["tree", "--all"]] {
        let o = repo.run(&repo.path(), args, &[], "");
        assert!(o.status.success(), "{args:?}: {}", err(&o));
    }
    assert!(!repo.home().join("phrase").exists());
}
