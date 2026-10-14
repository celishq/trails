//! CIP-01, "The mirror", and CIP-02, "Mirror".

mod support;

use celis_nostr::{Account, Note};
use std::path::Path;
use support::{err, now, out, step_tags, Repo};

/// The records in the `.json` files of `folder`.
fn records(folder: &Path) -> Vec<Note> {
    std::fs::read_dir(folder)
        .into_iter()
        .flatten()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .filter_map(|p| serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok())
        .collect()
}

fn ids(folder: &Path) -> Vec<String> {
    let mut ids: Vec<String> = records(folder).into_iter().map(|n| n.id).collect();
    ids.sort();
    ids
}

/// Runs `celis args` in the main checkout and returns its stdout; it must succeed.
fn run(repo: &Repo, args: &[&str]) -> String {
    let o = repo.run(&repo.path(), args, &[], "");
    assert!(o.status.success(), "{args:?}: {}", err(&o));
    out(&o)
}

fn mirror(repo: &Repo) -> std::path::PathBuf {
    repo.path().join(".celis/trails")
}

/// A step of the repo, signed by a stranger, written into the mirror folder alone.
fn strangers_step(repo: &Repo) -> Note {
    let stranger = Account::new(&[2u8; 32]).unwrap();
    let tags = step_tags((&repo.project_id(), "repo"));
    let note = Note::write(&stranger, 1111u16, now(), tags, "Ben's step").unwrap();
    std::fs::create_dir_all(mirror(repo)).unwrap();
    let name = format!("{}-{}.json", note.created_at, note.id);
    std::fs::write(
        mirror(repo).join(name),
        serde_json::to_string(&note).unwrap(),
    )
    .unwrap();
    note
}

/// `mirror` creates `.celis/trails/` and `.celis/README.md`, and copies the project's records out of the
/// store (`spec/02-cli.md`, "Mirror").
#[test]
fn mirror_creates_the_folder_and_copies_the_projects_records_out() {
    let repo = Repo::new();
    run(&repo, &["step", "add", "--", "Keep prices in cents."]);
    let shown = run(&repo, &["mirror"]);
    let copied = ids(&mirror(&repo));
    assert!(!copied.is_empty());
    assert_eq!(copied, ids(&repo.store().join("shared")));
    assert_eq!(
        shown.trim(),
        format!(".celis/trails · {} out · 0 in", copied.len())
    );
    let readme = std::fs::read_to_string(repo.path().join(".celis/README.md")).unwrap();
    assert!(readme.contains("celis"), "{readme}");
    let listed = std::fs::read_to_string(repo.home().join("mirrors")).unwrap();
    assert_eq!(listed.trim(), repo.project_id());
}

/// A second `mirror` copies nothing, and `--json` prints the folder and both counts
/// (`spec/02-cli.md`, "Mirror").
#[test]
fn a_second_mirror_copies_nothing() {
    let repo = Repo::new();
    run(&repo, &["step", "add", "--", "Keep prices in cents."]);
    run(&repo, &["mirror"]);
    let shown = run(&repo, &["mirror", "--json"]);
    let json: serde_json::Value = serde_json::from_str(shown.trim()).unwrap();
    assert_eq!(
        json,
        serde_json::json!({"folder": ".celis/trails", "out": 0, "in": 0})
    );
    let listed = std::fs::read_to_string(repo.home().join("mirrors")).unwrap();
    assert_eq!(listed.lines().count(), 1, "{listed}");
}

/// A mirror holds no `me` decision and no record of another project, and holds a status record by the
/// step it names (`spec/01-decisions.md`, "The mirror").
#[test]
fn a_mirror_holds_only_the_projects_records_and_no_me_decision() {
    let repo = Repo::new();
    let mine = run(
        &repo,
        &[
            "decide",
            "--scope",
            "me",
            "--",
            "Write plain commit messages.",
        ],
    );
    let kept = run(&repo, &["decide", "--", "Keep prices in cents."]);
    let step = run(&repo, &["step", "add", "--", "Round the price down."]);
    let step = step.lines().next().unwrap().to_string();
    let started = run(&repo, &["step", "start", &step]);
    let other = repo.other("crm");
    let elsewhere = repo.run(&other, &["decide", "--", "Use the shared table."], &[], "");
    run(&repo, &["mirror"]);
    let held = ids(&mirror(&repo));
    assert!(!held.contains(&mine.trim().to_string()), "{held:?}");
    assert!(
        !held.contains(&out(&elsewhere).trim().to_string()),
        "{held:?}"
    );
    assert!(held.contains(&kept.trim().to_string()), "{held:?}");
    assert!(held.contains(&step), "{held:?}");
    let start = records(&mirror(&repo))
        .into_iter()
        .any(|n| n.kind == 1985 && n.tag("e") == Some(step.as_str()));
    assert!(start, "{started}");
}

/// When the mirror is on, each record the mirror holds is written to it as well
/// (`spec/01-decisions.md`, "The mirror").
#[test]
fn when_the_mirror_is_on_a_new_record_is_written_to_it() {
    let repo = Repo::new();
    run(&repo, &["mirror"]);
    let kept = run(&repo, &["decide", "--", "Keep prices in cents."]);
    let mine = run(
        &repo,
        &[
            "decide",
            "--scope",
            "me",
            "--",
            "Write plain commit messages.",
        ],
    );
    let held = ids(&mirror(&repo));
    assert_eq!(held, vec![kept.trim().to_string()], "{mine}");
}

/// A mirror folder that this machine did not turn on is not read, and nothing is copied from it
/// (`spec/01-decisions.md`, "The mirror").
#[test]
fn a_mirror_that_is_not_on_is_not_read() {
    let repo = Repo::new();
    run(&repo, &["decide", "--", "Keep prices in cents."]);
    let note = strangers_step(&repo);
    let shown = run(&repo, &["tree", "--all"]);
    assert!(!shown.contains("Ben's step"), "{shown}");
    assert!(!ids(&repo.store().join("shared")).contains(&note.id));
    assert_eq!(ids(&mirror(&repo)), vec![note.id]);
}

/// When the mirror is on, a reader reads it, and copies into the store each record the store does not
/// have (`spec/01-decisions.md`, "The mirror"; `spec/02-cli.md`, "Mirror").
#[test]
fn when_the_mirror_is_on_its_records_are_read_and_copied_in() {
    let repo = Repo::new();
    run(&repo, &["mirror"]);
    let note = strangers_step(&repo);
    let shown = run(&repo, &["tree", "--all"]);
    assert!(shown.contains("Ben's step"), "{shown}");
    assert!(ids(&repo.store().join("shared")).contains(&note.id));
    let other = Repo::new();
    std::fs::create_dir_all(mirror(&other)).unwrap();
    let theirs = strangers_step(&other);
    let shown = run(&other, &["mirror"]);
    assert_eq!(shown.trim(), ".celis/trails · 0 out · 1 in");
    assert!(ids(&other.store().join("shared")).contains(&theirs.id));
}

/// A worktree with no `.celis/trails/` writes no mirror, though its project is mirrored
/// (`spec/01-decisions.md`, "The mirror").
#[test]
fn a_checkout_without_the_folder_writes_no_mirror() {
    let repo = Repo::new();
    let tree = repo.worktree("feat-a");
    run(&repo, &["mirror"]);
    let o = repo.run(&tree, &["decide", "--", "Keep prices in cents."], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    assert!(!tree.join(".celis").exists());
}

/// When the top folder has `.celis/trails/` and the mirror is not on, `prompt` asks, once a session, to
/// ask the person whether to sync, and names `celis mirror` (`spec/02-cli.md`, "Prompt").
#[test]
fn prompt_asks_to_sync_a_project_with_a_mirror_that_is_not_on() {
    let repo = Repo::new();
    run(&repo, &["decide", "--", "Keep prices in cents."]);
    strangers_step(&repo);
    let hook = |session: &str| {
        let json =
            serde_json::json!({"session_id": session, "prompt": "Fix the cart."}).to_string();
        repo.run(&repo.path(), &["prompt"], &[], &json)
    };
    let first = err(&hook("s1"));
    assert!(first.contains("celis mirror"), "{first}");
    assert!(first.contains("sync"), "{first}");
    let second = err(&hook("s1"));
    assert!(!second.contains("celis mirror"), "{second}");
    run(&repo, &["mirror"]);
    let on = err(&hook("s2"));
    assert!(!on.contains("celis mirror"), "{on}");
}

/// Without `.celis/trails/`, `prompt` does not ask to sync (`spec/02-cli.md`, "Prompt").
#[test]
fn prompt_does_not_ask_to_sync_a_project_with_no_mirror() {
    let repo = Repo::new();
    run(&repo, &["decide", "--", "Keep prices in cents."]);
    let json = serde_json::json!({"session_id": "s1", "prompt": "Fix the cart."}).to_string();
    let said = err(&repo.run(&repo.path(), &["prompt"], &[], &json));
    assert!(!said.contains("celis mirror"), "{said}");
}

/// `mirror --off` takes the project out of `$CELIS_HOME/mirrors` and keeps the folder; then nothing is
/// written to the folder or read from it (`spec/02-cli.md`, "Mirror").
#[test]
fn mirror_off_stops_the_sync_and_keeps_the_folder() {
    let repo = Repo::new();
    run(&repo, &["mirror"]);
    let shown = run(&repo, &["mirror", "--off"]);
    assert_eq!(shown.trim(), ".celis/trails · off");
    let json = run(&repo, &["mirror", "--off", "--json"]);
    let json: serde_json::Value = serde_json::from_str(json.trim()).unwrap();
    assert_eq!(
        json,
        serde_json::json!({"folder": ".celis/trails", "off": true})
    );
    let listed = std::fs::read_to_string(repo.home().join("mirrors")).unwrap();
    assert!(!listed.contains(&repo.project_id()), "{listed}");
    assert!(repo.path().join(".celis/README.md").exists());
    run(&repo, &["decide", "--", "Keep prices in cents."]);
    assert!(ids(&mirror(&repo)).is_empty());
    let note = strangers_step(&repo);
    let tree = run(&repo, &["tree", "--all"]);
    assert!(!tree.contains("Ben's step"), "{tree}");
    assert!(!ids(&repo.store().join("shared")).contains(&note.id));
}

/// With no store, `prompt` still asks to sync a project whose mirror is not on, and creates no store
/// (`spec/02-cli.md`, "Prompt").
#[test]
fn prompt_with_no_store_asks_to_sync() {
    let repo = Repo::new();
    std::fs::create_dir_all(mirror(&repo)).unwrap();
    let json = serde_json::json!({"session_id": "s1", "prompt": "Fix the cart."}).to_string();
    let said = err(&repo.run(&repo.path(), &["prompt"], &[], &json));
    assert!(said.contains("celis mirror"), "{said}");
    assert!(!repo.store().exists());
}

/// The README of `.celis/` says how to install celis (`spec/02-cli.md`, "Mirror").
#[test]
fn the_readme_says_how_to_install_celis() {
    let repo = Repo::new();
    run(&repo, &["mirror"]);
    let readme = std::fs::read_to_string(repo.path().join(".celis/README.md")).unwrap();
    assert!(readme.contains("https://celis.ai/install.sh"), "{readme}");
}
