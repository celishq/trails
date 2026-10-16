//! CIP-01, "The store".

mod support;

use celis_nostr::Note;
use celis_trails::Store;
use support::{account, err, out, Repo, PHRASE};

/// A command opens the store in `$CELIS_HOME`, so a trail written in one folder lists in another, inside
/// git or not (`spec/01-decisions.md`, "The store").
#[test]
fn a_command_must_open_the_store_in_celis_home_whatever_the_working_folder() {
    let repo = Repo::new();
    let text = "Before you run make, set CI=1.";
    let wrote = repo.run(&repo.path(), &["step", "add", "--", text], &[], "");
    assert!(wrote.status.success(), "{}", err(&wrote));
    let tree = repo.worktree("feat-a");
    let listed = repo.run(&tree, &["grep", "make"], &[], "");
    assert!(out(&listed).contains(text), "{}", out(&listed));
    let outside = tempfile::tempdir().unwrap();
    let elsewhere = repo.run(outside.path(), &["step", "add", "--", "Run make."], &[], "");
    assert!(elsewhere.status.success(), "{}", err(&elsewhere));
    for folder in [tree.as_path(), outside.path()] {
        assert!(!folder.join(".celis").exists(), "{}", folder.display());
    }
    let written = std::fs::read_dir(repo.store().join("shared"))
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .path()
                .extension()
                .is_some_and(|x| x == "json")
        })
        .count();
    assert!(written >= 2, "{written}");
    assert!(!repo.path().join(".celis").exists());
}

/// With no store, `decide` and `step add` create it (`spec/01-decisions.md`, "The store").
#[test]
fn with_no_store_decide_and_step_add_create_it() {
    for args in [
        &["decide", "--", "Keep prices in cents."][..],
        &["step", "add", "--", "work"],
    ] {
        let repo = Repo::plain();
        let o = repo.run(&repo.path(), args, &[], "");
        assert!(o.status.success(), "{args:?}: {}", err(&o));
        assert!(repo.store().join("shared").is_dir(), "{args:?}");
    }
}

/// A record is written to `<created_at>-<id>.json` holding the record as NIP-01 JSON
/// (`spec/01-decisions.md`, "The store").
#[test]
fn a_record_must_be_written_to_a_file_named_by_its_time_and_id() {
    let repo = Repo::new();
    let store = Store::open(&repo.home()).ok().unwrap();
    let record = Note::write(&account(), 1111u16, 1759400060, vec![], "first").unwrap();
    let path = store.write(&record).unwrap();
    assert_eq!(
        path.file_name().unwrap().to_str().unwrap(),
        format!("{}-{}.json", record.created_at, record.id)
    );
    let read: Note = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(read, record);
}

/// A record file does not change once written, and a file still being written is not seen
/// (`spec/01-decisions.md`, "The store").
#[test]
fn a_record_file_must_not_change_once_written() {
    let repo = Repo::new();
    let store = Store::open(&repo.home()).ok().unwrap();
    let record = Note::write(&account(), 1111u16, 1759400060, vec![], "first").unwrap();
    let shared = repo.store().join("shared");
    let path = shared.join(format!("{}-{}.json", record.created_at, record.id));
    std::fs::write(&path, "already here").unwrap();
    store.write(&record).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "already here");

    let other = Note::write(&account(), 1111u16, 1759400070, vec![], "second").unwrap();
    std::fs::write(
        shared.join(format!(".{}-{}.json.1", other.created_at, other.id)),
        serde_json::to_string(&other).unwrap(),
    )
    .unwrap();
    assert!(!store.records().contains(&other));
}

/// A reader never sees a record file half written, however large the record
/// (`spec/01-decisions.md`, "The store").
#[test]
fn a_record_file_must_never_be_seen_half_written() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let repo = Repo::new();
    let store = Store::open(&repo.home()).ok().unwrap();
    let shared = repo.store().join("shared");
    let content = "x".repeat(32 * 1024 * 1024);
    let record = Note::write(&account(), 1111u16, 1759400060, vec![], &content).unwrap();
    let done = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let watch = scope.spawn(|| {
            while !done.load(Ordering::SeqCst) {
                for entry in std::fs::read_dir(&shared).unwrap().flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|x| x == "json") {
                        let text = std::fs::read_to_string(&path).unwrap_or_default();
                        let note: Note = serde_json::from_str(&text)
                            .unwrap_or_else(|e| panic!("a named file is half written: {e}"));
                        assert_eq!(note.id, record.id);
                    }
                }
            }
        });
        store.write(&record).unwrap();
        done.store(true, Ordering::SeqCst);
        watch.join().unwrap()
    });
    assert!(store.records().contains(&record));
}

/// A reader skips a file that does not parse or whose record does not verify
/// (`spec/01-decisions.md`, "The store").
#[test]
fn a_reader_must_skip_a_file_that_does_not_parse_or_verify() {
    let repo = Repo::new();
    let store = Store::open(&repo.home()).ok().unwrap();
    let good = Note::write(&account(), 1111u16, 1759400060, vec![], "good").unwrap();
    store.write(&good).unwrap();

    let shared = repo.store().join("shared");
    std::fs::write(shared.join("1759400061-garbage.json"), "{ not json").unwrap();
    let mut forged = Note::write(&account(), 1111u16, 1759400062, vec![], "forged").unwrap();
    forged.content = "changed after signing".into();
    std::fs::write(
        shared.join(format!("1759400062-{}.json", forged.id)),
        serde_json::to_string(&forged).unwrap(),
    )
    .unwrap();

    assert_eq!(store.records(), vec![good]);
}

/// No record file carries the phrase or the account's secret
/// (`spec/01-decisions.md`, "The store").
#[test]
fn no_record_file_must_carry_the_accounts_secret() {
    let repo = Repo::plain();
    let wrote = repo.run(
        &repo.path(),
        &["step", "add", "--", "Before you run make, set CI=1."],
        &[],
        "",
    );
    assert!(wrote.status.success(), "{}", err(&wrote));
    let secret: String = account()
        .secret()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    for entry in std::fs::read_dir(repo.store().join("shared")).unwrap() {
        let text = std::fs::read_to_string(entry.unwrap().path()).unwrap();
        assert!(!text.contains(PHRASE) && !text.contains(&secret));
    }
}

/// With no store, `grep` answers as an empty store and creates nothing, inside git or not
/// (`spec/01-decisions.md`, "The store").
#[test]
fn grep_with_no_store_must_answer_empty_and_create_nothing() {
    for repo in [Repo::new(), Repo::plain()] {
        let o = repo.run(&repo.path(), &["grep", "x"], &[], "");
        assert!(o.status.success(), "{}", err(&o));
        assert_eq!(out(&o).trim(), "No line matches.");
        assert!(!repo.store().exists());
    }
}

/// With no store, `tree` answers as an empty store and creates nothing (`spec/01-decisions.md`, "The
/// store").
#[test]
fn tree_with_no_store_must_answer_empty_and_create_nothing() {
    let repo = Repo::new();
    let o = repo.run(&repo.path(), &["tree"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).starts_with("No active trails."), "{}", out(&o));
    assert!(!repo.store().exists());
}

/// With no store, a command that writes creates it (`spec/01-decisions.md`, "The store").
#[test]
fn with_no_store_every_other_command_must_create_it() {
    let repo = Repo::new();
    let o = repo.run(
        &repo.path(),
        &["step", "add", "--", "Show a tier."],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    assert!(repo.store().join("shared").is_dir());
}
