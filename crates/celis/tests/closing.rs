//! CIP-12, "Closing".

mod support;

use celis_wire::{Account, Note};
use support::{account, err, now, out, put, Repo, DAY};

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

/// What `read` prints in the repo: the open steps among the rest.
fn ready(repo: &Repo) -> String {
    let o = repo.run(&repo.path(), &["read"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    out(&o)
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
/// (`spec/trails/12-steps.md`, "Closing").
#[test]
fn a_step_is_open_until_a_done_names_it_and_done_carries_its_proof() {
    let repo = Repo::new();
    let by = &account();
    let p = step(&repo, by, "work.", &[], now() - 4 * DAY).id;
    let q = step(&repo, by, "other.", &[], now() - 4 * DAY + 1).id;
    assert!(ready(&repo).contains(&p[..8]));
    let o = run(&repo, &["done", &p, "--", "proof"]);
    assert!(o.status.success(), "{}", err(&o));
    let all = ready(&repo);
    assert!(!all.contains(&p[..8]) && all.contains(&q[..8]), "{all}");
    let done = done_records(&repo);
    let note = done
        .iter()
        .find(|n| n.tag("e") == Some(p.as_str()))
        .unwrap();
    assert_eq!(note.content, "proof");
}

/// A reader ignores a `done` record with no content, so the step stays open (`spec/trails/12-steps.md`,
/// "Closing").
#[test]
fn a_reader_ignores_a_done_record_with_no_content() {
    let repo = Repo::plain();
    let by = &account();
    let p = step(&repo, by, "work", &[], now() - 4 * DAY);
    let tags: Vec<Vec<String>> = vec![
        vec!["L".into(), "celis.step".into()],
        vec!["l".into(), "done".into(), "celis.step".into()],
        vec!["e".into(), p.id.clone()],
    ];
    put(
        &repo,
        &Note::write(by, 1985u16, now() - 3 * DAY, tags, "").unwrap(),
    );
    let all = ready(&repo);
    assert!(all.contains(&p.id[..8]), "{all}");
}

/// `done` refuses an empty proof as a usage error (`spec/trails/12-steps.md`, "The commands").
#[test]
fn done_refuses_an_empty_proof_as_a_usage_error() {
    let repo = Repo::plain();
    let by = &account();
    let p = step(&repo, by, "work", &[], now() - 4 * DAY);
    let o = run(&repo, &["done", &p.id[..8], "--", ""]);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
}
