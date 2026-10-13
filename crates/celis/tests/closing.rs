//! CIP-03, "Closing".

mod support;

use celis_nostr::{Account, Note};
use support::{account, err, now, objects, out, put, Repo};

fn step(repo: &Repo, by: &Account, text: &str, extra: &[(&str, &str)], at: u64) -> Note {
    let anchor = format!("project:{}", repo.project_id());
    let mut tags: Vec<Vec<String>> = vec![
        vec!["I".into(), anchor.clone()],
        vec!["K".into(), "project".into()],
        vec!["i".into(), anchor],
        vec!["k".into(), "project".into()],
        vec!["project".into(), "repo".into()],
        vec!["L".into(), "celis.step".into()],
        vec!["l".into(), "step".into(), "celis.step".into()],
    ];
    for (name, value) in extra {
        tags.push(vec![name.to_string(), value.to_string()]);
    }
    let note = Note::write(by, 1111u16, at, tags, text).unwrap();
    put(repo, &note);
    note
}

fn run(repo: &Repo, args: &[&str]) -> std::process::Output {
    let mut all = vec!["step"];
    all.extend(args);
    repo.run(&repo.path(), &all, &[], "")
}

/// The mark `tree --all` prints before the step `id`: `●` while it is open.
fn mark(repo: &Repo, id: &str) -> String {
    let o = repo.run(&repo.path(), &["tree", "--all"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let all = out(&o);
    let line = all.lines().find(|l| l.contains(&id[..8])).unwrap_or("");
    line.chars().take(1).collect()
}

fn done_records(repo: &Repo) -> Vec<Note> {
    std::fs::read_dir(repo.store().join("shared"))
        .unwrap()
        .filter_map(|e| std::fs::read_to_string(e.unwrap().path()).ok())
        .filter_map(|t| serde_json::from_str::<Note>(&t).ok())
        .filter(|n| n.kind == 1985 && n.tag("l") == Some("done"))
        .collect()
}

/// A step is open until a `done` names it, and `done` carries its proof as content
/// (`spec/trails/03-steps.md`, "Closing").
#[test]
fn a_step_is_open_until_a_done_names_it_and_done_carries_its_proof() {
    let repo = Repo::new();
    let by = &account();
    let p = step(&repo, by, "work.", &[], now() - 4 * 60).id;
    let q = step(&repo, by, "other.", &[], now() - 4 * 60 + 1).id;
    assert_eq!(mark(&repo, &p), "●");
    let o = run(&repo, &["done", &p, "--as-planned", "--", "proof"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(mark(&repo, &p), "✓");
    assert_eq!(mark(&repo, &q), "●");
    let done = done_records(&repo);
    let note = done
        .iter()
        .find(|n| n.tag("e") == Some(p.as_str()))
        .unwrap();
    assert_eq!(note.content, "proof");
}

/// A reader ignores a `done` record with no content, so the step stays open (`spec/trails/03-steps.md`,
/// "Closing").
#[test]
fn a_reader_ignores_a_done_record_with_no_content() {
    let repo = Repo::plain();
    let by = &account();
    let p = step(&repo, by, "work", &[], now() - 4 * 60);
    let tags: Vec<Vec<String>> = vec![
        vec!["L".into(), "celis.step".into()],
        vec!["l".into(), "done".into(), "celis.step".into()],
        vec!["e".into(), p.id.clone()],
    ];
    put(
        &repo,
        &Note::write(by, 1985u16, now() - 3 * 60, tags, "").unwrap(),
    );
    assert_eq!(mark(&repo, &p.id), "●");
}

/// `done` refuses an empty proof as a usage error (`spec/trails/03-steps.md`, "The commands").
#[test]
fn done_refuses_an_empty_proof_as_a_usage_error() {
    let repo = Repo::plain();
    let by = &account();
    let p = step(&repo, by, "work", &[], now() - 4 * 60);
    let o = run(&repo, &["done", &p.id[..8], "--as-planned", "--", ""]);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
}

/// `done` asks what changed from the step's title, or that the work went as planned, and refuses neither
/// or both as a usage error; the done record carries the change (`spec/trails/03-steps.md`, "Closing").
#[test]
fn done_says_what_changed_or_that_it_went_as_planned() {
    let repo = Repo::new();
    let by = &account();
    let moved = step(&repo, by, "moved", &[], now() - 4 * 60).id;
    let kept = step(&repo, by, "kept", &[], now() - 4 * 60 + 1).id;
    let neither = run(&repo, &["done", &moved[..8], "--", "p"]);
    assert_eq!(neither.status.code(), Some(2), "{}", err(&neither));
    let both = [
        "done",
        &moved[..8],
        "--as-planned",
        "--changed",
        "x",
        "--",
        "p",
    ];
    assert_eq!(run(&repo, &both).status.code(), Some(2));
    assert!(done_records(&repo).is_empty());
    let changed = [
        "done",
        &moved[..8],
        "--changed",
        "Read prints each step's own line.",
        "--",
        "p",
    ];
    let o = run(&repo, &changed);
    assert!(o.status.success(), "{}", err(&o));
    let o = run(&repo, &["done", &kept[..8], "--as-planned", "--", "p"]);
    assert!(o.status.success(), "{}", err(&o));
    let change = |id: &str| objects(&repo, &["tree", &id[..8]])[0]["trail"]["changed"].clone();
    assert_eq!(change(&moved), "Read prints each step's own line.");
    assert!(change(&kept).is_null());
}
