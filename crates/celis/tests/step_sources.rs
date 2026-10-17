//! CIP-03, "Sources".

mod support;

use celis_nostr::{Account, Note};
use std::process::Output;
use support::{account, err, now, out, put, Repo};

fn run(repo: &Repo, args: &[&str]) -> Output {
    let mut all = vec!["step"];
    all.extend(args);
    repo.run(&repo.path(), &all, &[], "")
}

fn store_step(repo: &Repo, by: &Account, text: &str, at: u64) -> Note {
    let anchor = format!("project:{}", repo.project_id());
    let tags: Vec<Vec<String>> = vec![
        vec!["I".into(), anchor.clone()],
        vec!["K".into(), "project".into()],
        vec!["i".into(), anchor],
        vec!["k".into(), "project".into()],
        vec!["project".into(), "repo".into()],
        vec!["L".into(), "celis.step".into()],
        vec!["l".into(), "step".into(), "celis.step".into()],
    ];
    let note = Note::write(by, 1111u16, at, tags, text).unwrap();
    put(repo, &note);
    note
}

/// The ids, first 8 characters, of the trails `grep` prints in the repo.
fn trails(repo: &Repo) -> Vec<String> {
    let o = repo.run(&repo.path(), &["grep", " trail "], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    support::ids(&out(&o), "trail")
}

fn steps(repo: &Repo) -> usize {
    std::fs::read_dir(repo.store().join("shared"))
        .unwrap()
        .filter_map(|e| std::fs::read_to_string(e.unwrap().path()).ok())
        .filter_map(|t| serde_json::from_str::<Note>(&t).ok())
        .filter(|n| n.kind == 1111 && n.tag("l") == Some("step"))
        .count()
}

/// A reader reads steps from the store only, so a project in git ignores its `@todo` lines
/// (`spec/03-steps.md`, "Sources").
#[test]
fn a_reader_reads_steps_from_the_store_only() {
    let repo = Repo::new();
    let by = &account();
    let kept = store_step(&repo, by, "in the store", now() - 3 * 60);
    repo.write("a.rs", "// @todo #1 in the code\n");
    repo.git(&["add", "."]);
    assert_eq!(trails(&repo), [kept.id[..8].to_string()]);
}

/// A reader ignores a `done` that names its step with a `step` tag
/// (`spec/01-records.md`, "Records").
#[test]
fn a_reader_ignores_a_done_that_names_its_step_with_a_step_tag() {
    let repo = Repo::new();
    let by = &account();
    let kept = store_step(&repo, by, "work", now() - 3 * 60);
    let tags: Vec<Vec<String>> = vec![
        vec!["L".into(), "celis.step".into()],
        vec!["l".into(), "done".into(), "celis.step".into()],
        vec!["step".into(), kept.id.clone()],
    ];
    put(
        &repo,
        &Note::write(by, 1985u16, now() - 60, tags, "proof").unwrap(),
    );
    assert_eq!(trails(&repo), [kept.id[..8].to_string()]);
}

/// `add` writes a step in a project in git as outside it (`spec/03-steps.md`, "Sources").
#[test]
fn add_writes_a_step_in_a_project_in_git() {
    let repo = Repo::new();
    let o = run(&repo, &["add", "--", "undo it."]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(trails(&repo).len(), 1);
}

/// `add` writes no step when an open step of the same project and `under` has the same text, and prints
/// that step's id (`spec/02-cli.md`, "Step").
#[test]
fn add_must_not_write_a_step_an_open_one_of_the_project_already_states() {
    let repo = Repo::new();
    let first = run(&repo, &["add", "--", "undo it."]);
    assert!(first.status.success(), "{}", err(&first));
    let again = run(&repo, &["add", "--at", "a.rs:9", "--", "undo it."]);
    assert!(again.status.success(), "{}", err(&again));
    assert_eq!(out(&again), out(&first));
    assert_eq!(steps(&repo), 1);
    let outside = tempfile::tempdir().unwrap();
    let elsewhere = repo.run(outside.path(), &["step", "add", "--", "undo it."], &[], "");
    assert!(elsewhere.status.success(), "{}", err(&elsewhere));
    assert_ne!(out(&elsewhere), out(&first));
    let a = out(&run(&repo, &["add", "--", "trail a"]))
        .trim()
        .to_string();
    let b = out(&run(&repo, &["add", "--", "trail b"]))
        .trim()
        .to_string();
    let in_a = run(&repo, &["add", "--under", &a[..8], "--", "write the tests"]);
    let in_b = run(&repo, &["add", "--under", &b[..8], "--", "write the tests"]);
    assert_ne!(out(&in_a), out(&in_b));
}
