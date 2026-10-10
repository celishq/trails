//! CIP-12, "The commands".

mod support;

use celis_trails::Project;
use celis_wire::{Account, Note};
use serde_json::Value;
use std::process::Output;
use support::{account, err, now, out, put, Repo, DAY};

fn step(by: &Account, project: (&str, &str), text: &str, extra: &[(&str, &str)], at: u64) -> Note {
    let anchor = format!("project:{}", project.0);
    let mut tags: Vec<Vec<String>> = vec![
        vec!["I".into(), anchor.clone()],
        vec!["K".into(), "project".into()],
        vec!["i".into(), anchor],
        vec!["k".into(), "project".into()],
        vec!["project".into(), project.1.into()],
        vec!["L".into(), "celis.step".into()],
        vec!["l".into(), "step".into(), "celis.step".into()],
    ];
    for (name, value) in extra {
        tags.push(vec![name.to_string(), value.to_string()]);
    }
    Note::write(by, 1111u16, at, tags, text).unwrap()
}

fn record(repo: &Repo, by: &Account, label: &str, id: &str, at: u64) {
    let tags: Vec<Vec<String>> = vec![
        vec!["L".into(), "celis.step".into()],
        vec!["l".into(), label.into(), "celis.step".into()],
        vec!["e".into(), id.into()],
    ];
    let content = if label == "done" { "proof" } else { "" };
    put(repo, &Note::write(by, 1985u16, at, tags, content).unwrap());
}

fn run(repo: &Repo, args: &[&str]) -> Output {
    let mut all = vec!["step"];
    all.extend(args);
    repo.run(&repo.path(), &all, &[], "")
}

fn refusal(o: &Output, code: i32, name: &str) {
    assert_eq!(o.status.code(), Some(code), "{}", err(o));
    let object: Value =
        serde_json::from_str(err(o).trim()).unwrap_or_else(|_| panic!("{}", err(o)));
    assert_eq!(object["error"], name, "{}", err(o));
}

/// `step add` refuses empty text as a usage error; `done` refuses an unknown id with `no-step` and a closed step with `closed`, and
/// takes the first 8 characters of an id (`spec/trails/12-steps.md`, "The commands").
#[test]
fn step_add_refuses_empty_text_and_the_other_commands_refuse_unknown_and_closed_ids() {
    let repo = Repo::plain();
    let by = &account();
    let id = repo.project_id();
    let open = step(by, (&id, "repo"), "open", &[], now() - 3 * DAY);
    let shut = step(by, (&id, "repo"), "shut", &[], now() - 3 * DAY + 1);
    put(&repo, &open);
    put(&repo, &shut);
    record(&repo, by, "done", &shut.id, now() - 2 * DAY);
    refusal(&run(&repo, &["add", "--json", "--", ""]), 2, "usage");
    let nowhere = "0".repeat(64);
    let gone = ["done", "--json", &nowhere, "--", "p"];
    let closed = ["done", "--json", &shut.id[..8], "--", "p"];
    refusal(&run(&repo, &gone), 1, "no-step");
    refusal(&run(&repo, &closed), 1, "closed");
    let o = run(&repo, &["done", &open.id[..8], "--", "proof"]);
    assert!(o.status.success(), "{}", err(&o));
}

/// `read` prints each open step of the working folder's project oldest first, as
/// `<id, first 8> [<project name>] <text>`, then ` · <at>` when it has one, and with `--json` the spec
/// object (`spec/trails/12-steps.md`, "The commands").
#[test]
fn read_prints_each_open_step_as_the_line_and_object_the_spec_gives() {
    let repo = Repo::plain();
    let by = &account();
    let other = repo.path().with_file_name("elsewhere");
    std::fs::create_dir_all(&other).unwrap();
    let home = (repo.project_id(), "repo".to_string());
    let away = (Project::at(&other).id, "elsewhere".to_string());
    let t = now() - 5 * DAY;
    let trail = step(by, (&home.0, &home.1), "the trail", &[], t - 1);
    let under = [("under", trail.id.as_str())];
    let at = [("under", trail.id.as_str()), ("at", "a.rs:1")];
    let second = step(by, (&home.0, &home.1), "second", &at, t + 10);
    let first = step(by, (&home.0, &home.1), "first", &under, t);
    let elsewhere = step(by, (&away.0, &away.1), "elsewhere", &under, t + 20);
    for n in [&trail, &second, &first, &elsewhere] {
        put(&repo, n);
    }
    let o = repo.run(&repo.path(), &["read"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&o);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(
        lines,
        vec![
            format!("{} [repo] the trail", &trail.id[..8]),
            format!("{} [repo] first", &first.id[..8]),
            format!("{} [repo] second · a.rs:1", &second.id[..8]),
        ],
        "{text}"
    );
    let o = repo.run(&repo.path(), &["read", "--json"], &[], "");
    let json = out(&o);
    let line: Value = serde_json::from_str(json.lines().nth(1).unwrap()).unwrap();
    let object = &line["step"];
    assert_eq!(object["id"], first.id.as_str());
    assert_eq!(object["project"], home.0.as_str());
    assert!(object.get("task").is_none(), "{object}");
    assert_eq!(object["under"], trail.id.as_str());
    assert_eq!(object["content"], "first");
    assert!(object.get("holder").is_none(), "{object}");
    assert!(object.get("agent").is_none(), "{object}");
}

fn stored(repo: &Repo) -> Vec<Note> {
    std::fs::read_dir(repo.store().join("shared"))
        .unwrap()
        .filter_map(|e| std::fs::read_to_string(e.unwrap().path()).ok())
        .filter_map(|t| serde_json::from_str::<Note>(&t).ok())
        .collect()
}

fn written(repo: &Repo, label: &str, id: &str) -> Vec<Note> {
    let mut found: Vec<Note> = stored(repo)
        .into_iter()
        .filter(|n| n.kind == 1985 && n.tag("l") == Some(label) && n.tag("e") == Some(id))
        .collect();
    found.sort_by(|a, b| a.id.cmp(&b.id));
    found
}

/// `add` and `done` print the id they wrote, and `{"id"}` with `--json`
/// (`spec/trails/12-steps.md`, "The commands").
#[test]
fn the_step_commands_print_the_ids_they_wrote() {
    let repo = Repo::plain();
    let by = &account();
    let id = repo.project_id();
    let at = now() - 3 * DAY;
    let notes: Vec<Note> = ["a", "b", "c", "d"]
        .iter()
        .enumerate()
        .map(|(i, text)| step(by, (&id, "repo"), text, &[], at + i as u64))
        .collect();
    for n in &notes {
        put(&repo, n);
    }

    let o = run(&repo, &["add", "--json", "--", "fresh"]);
    assert!(o.status.success(), "{}", err(&o));
    let added: Value = serde_json::from_str(out(&o).trim()).unwrap();
    let note = stored(&repo)
        .into_iter()
        .find(|n| n.content == "fresh")
        .unwrap();
    assert_eq!(added["id"], note.id.as_str());
    let o = run(&repo, &["add", "--", "plain"]);
    assert!(out(&o).contains(
        &stored(&repo)
            .into_iter()
            .find(|n| n.content == "plain")
            .unwrap()
            .id[..8]
    ));

    let o = run(&repo, &["done", "--json", &notes[0].id, "--", "p"]);
    assert!(o.status.success(), "{}", err(&o));
    let closed = written(&repo, "done", &notes[0].id);
    assert_eq!(closed.len(), 1);
    let printed: Value = serde_json::from_str(out(&o).trim()).unwrap();
    assert_eq!(printed["id"], closed[0].id.as_str());

    let o = run(&repo, &["done", &notes[1].id, "--", "p"]);
    assert!(o.status.success(), "{}", err(&o));
    let closed = written(&repo, "done", &notes[1].id);
    assert!(out(&o).contains(&closed[0].id[..8]), "{}", out(&o));
}

/// `celis step split` is an unknown command, a usage error (`spec/trails/12-steps.md`, "The commands").
#[test]
fn step_split_is_an_unknown_command_and_a_usage_error() {
    let repo = Repo::plain();
    let by = &account();
    let id = repo.project_id();
    let whole = step(by, (&id, "repo"), "big", &[], now() - 3 * DAY);
    put(&repo, &whole);
    let o = run(
        &repo,
        &["split", &whole.id, "--into", "one", "--into", "two"],
    );
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
}

/// `add` carries each `--file` and the `--check` as their tags (`spec/trails/12-steps.md`, "The commands").
#[test]
fn add_carries_each_file_and_the_check_as_their_tags() {
    let repo = Repo::plain();
    let o = run(
        &repo,
        &[
            "add", "--file", "a.rs", "--file", "b.rs", "--check", "make t", "--", "text",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let note = stored(&repo)
        .into_iter()
        .find(|n| n.id == out(&o).trim())
        .unwrap();
    let values = |name: &str| -> Vec<String> {
        note.tags
            .iter()
            .filter(|t| t[0] == name)
            .map(|t| t[1].clone())
            .collect()
    };
    assert_eq!(values("file"), ["a.rs", "b.rs"]);
    assert_eq!(values("check"), ["make t"]);
}
