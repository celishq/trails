//! CIP-02, "Step".

mod support;

use celis_nostr::{Account, Note};
use serde_json::Value;
use std::process::Output;
use support::{account, add, cli, err, now, out, put, Repo};

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
/// takes the first 8 characters of an id (`spec/02-cli.md`, "Step").
#[test]
fn step_add_refuses_empty_text_and_the_other_commands_refuse_unknown_and_closed_ids() {
    let repo = Repo::plain();
    let by = &account();
    let id = repo.project_id();
    let open = step(by, (&id, "repo"), "open", &[], now() - 3 * 60);
    let shut = step(by, (&id, "repo"), "shut", &[], now() - 3 * 60 + 1);
    put(&repo, &open);
    put(&repo, &shut);
    record(&repo, by, "done", &shut.id, now() - 2 * 60);
    refusal(&run(&repo, &["add", "--json", "--", ""]), 2, "usage");
    let nowhere = "0".repeat(64);
    let gone = ["done", "--json", &nowhere, "--as-planned", "--", "p"];
    let closed = ["done", "--json", &shut.id[..8], "--as-planned", "--", "p"];
    refusal(&run(&repo, &gone), 1, "no-step");
    refusal(&run(&repo, &closed), 1, "closed");
    let o = run(
        &repo,
        &["done", &open.id[..8], "--as-planned", "--", "proof"],
    );
    assert!(o.status.success(), "{}", err(&o));
}

/// A step prints with `--json` as the spec object (`spec/02-cli.md`, "Step").
#[test]
fn a_step_prints_as_the_line_and_object_the_spec_gives() {
    let repo = Repo::plain();
    let by = &account();
    let home = (repo.project_id(), "repo".to_string());
    let t = now() - 5 * 60;
    let trail = step(
        by,
        (&home.0, &home.1),
        "the trail",
        &[("at", "a.rs:1")],
        t - 1,
    );
    let under = [("under", trail.id.as_str())];
    let first = step(by, (&home.0, &home.1), "first", &under, t);
    for n in [&trail, &first] {
        put(&repo, n);
    }
    let o = repo.run(&repo.path(), &["tree", &trail.id[..8], "--json"], &[], "");
    let json = out(&o);
    let object = json
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .find_map(|v| v.get("step").cloned())
        .unwrap();
    assert_eq!(object["id"], first.id.as_str());
    assert_eq!(object["project"], home.0.as_str());
    assert!(object.get("task").is_none(), "{object}");
    assert_eq!(object["links"][0]["to"], trail.id.as_str());
    assert_eq!(object["links"][0]["kind"], "under");
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
/// (`spec/02-cli.md`, "Output and errors").
#[test]
fn the_step_commands_print_the_ids_they_wrote() {
    let repo = Repo::plain();
    let by = &account();
    let id = repo.project_id();
    let at = now() - 3 * 60;
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

    let o = run(
        &repo,
        &["done", "--json", &notes[0].id, "--as-planned", "--", "p"],
    );
    assert!(o.status.success(), "{}", err(&o));
    let closed = written(&repo, "done", &notes[0].id);
    assert_eq!(closed.len(), 1);
    let printed: Value = serde_json::from_str(out(&o).trim()).unwrap();
    assert_eq!(printed["id"], closed[0].id.as_str());

    let o = run(&repo, &["done", &notes[1].id, "--as-planned", "--", "p"]);
    assert!(o.status.success(), "{}", err(&o));
    let closed = written(&repo, "done", &notes[1].id);
    assert!(out(&o).contains(&closed[0].id[..8]), "{}", out(&o));
}

/// `celis step split` is an unknown command, a usage error.
#[test]
fn step_split_is_an_unknown_command_and_a_usage_error() {
    let repo = Repo::plain();
    let by = &account();
    let id = repo.project_id();
    let whole = step(by, (&id, "repo"), "big", &[], now() - 3 * 60);
    put(&repo, &whole);
    let o = run(
        &repo,
        &["split", &whole.id, "--into", "one", "--into", "two"],
    );
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
}

/// `add` carries each `--at` and the `--check` as their tags (`spec/02-cli.md`, "Step").
#[test]
fn add_carries_each_place_and_the_check_as_their_tags() {
    let repo = Repo::plain();
    let o = run(
        &repo,
        &[
            "add", "--at", "a.rs", "--at", "b.rs", "--check", "make t", "--", "text",
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
    assert_eq!(values("at"), ["a.rs", "b.rs"]);
    assert_eq!(values("check"), ["make t"]);
}

/// A reader counts only a step's `at` tags as places, not its old `file` tags
/// (`spec/01-records.md`, "Records").
#[test]
fn a_reader_ignores_a_steps_old_file_tags() {
    let repo = Repo::new();
    let project = (repo.project_id(), "repo".to_string());
    let extra = [("at", "a.rs"), ("file", "b.rs")];
    let old = step(
        &account(),
        (&project.0, &project.1),
        "old",
        &extra,
        now() - 60,
    );
    put(&repo, &old);
    let tree = support::objects(&repo, &["tree", &old.id[..8]]);
    assert_eq!(tree[0]["trail"]["places"], serde_json::json!(["a.rs"]));
}

/// `add` writes each `--at` as a path from the project's top folder
/// (`spec/02-cli.md`, "Step").
#[test]
fn add_writes_its_places_from_the_projects_top_folder() {
    let repo = Repo::new();
    repo.write("src/a.rs", "fn main() {}\n");
    let src = repo.path().join("src");
    let args = [
        "step", "add", "--at", "./a.rs:4", "--at", "../b.rs", "--", "text",
    ];
    let o = repo.run(&src, &args, &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let note = stored(&repo)
        .into_iter()
        .find(|n| n.id == out(&o).trim())
        .unwrap();
    let at: Vec<&str> = note
        .tags
        .iter()
        .filter(|t| t[0] == "at")
        .map(|t| t[1].as_str())
        .collect();
    assert_eq!(at, ["src/a.rs:4", "b.rs"]);
}

/// `done --pitfall` is a usage error: a pitfall is a `decide` after `done` (`spec/02-cli.md`,
/// "Commands").
#[test]
fn done_refuses_a_pitfall() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let step = add(&repo, &["--under", &trail[..8], "--", "loose"]);
    let args = [
        "step",
        "done",
        &step[..8],
        "--as-planned",
        "--pitfall",
        "wait",
        "--",
        "it passes",
    ];
    let o = cli(&repo, &args);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
}

/// `add` says when the step under an ask it goes under already holds an open step that has not
/// started, and still writes it; under an ask or a trail it says nothing.
#[test]
fn add_says_when_a_step_ahead_has_not_started() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let ask = add(&repo, &["--under", &trail[..8], "--", "the ask"]);
    let shape = add(&repo, &["--under", &ask[..8], "--", "a shape"]);
    let o = run(&repo, &["add", "--under", &ask[..8], "--", "another shape"]);
    assert_eq!(err(&o), "");
    let first = add(&repo, &["--under", &shape[..8], "--", "first"]);
    let o = run(&repo, &["add", "--under", &shape[..8], "--", "second"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(
        err(&o),
        format!(
            "celis: {} has not started; keep one step ahead: take it first, or drop it.\n",
            &first[..8]
        )
    );
}

/// `add` finds an open step of the same `under` and text, and names a step ahead that has not started,
/// also when their trail ended an hour ago.
#[test]
fn add_ignores_whether_the_trail_ended() {
    let repo = Repo::plain();
    let by = &account();
    let id = repo.project_id();
    let hour_ago = now() - 60 * 60;
    let trail = step(by, (&id, "repo"), "the trail", &[], hour_ago);
    let ask = step(
        by,
        (&id, "repo"),
        "the ask",
        &[("under", &trail.id)],
        hour_ago + 1,
    );
    let shape = step(
        by,
        (&id, "repo"),
        "a shape",
        &[("under", &ask.id)],
        hour_ago + 2,
    );
    let first = step(
        by,
        (&id, "repo"),
        "first",
        &[("under", &shape.id)],
        hour_ago + 3,
    );
    for note in [&trail, &ask, &shape, &first] {
        put(&repo, note);
    }
    let again = run(&repo, &["add", "--under", &shape.id[..8], "--", "first"]);
    assert!(again.status.success(), "{}", err(&again));
    assert_eq!(out(&again).trim(), first.id);
    let second = run(&repo, &["add", "--under", &shape.id[..8], "--", "second"]);
    assert!(second.status.success(), "{}", err(&second));
    assert_eq!(
        err(&second),
        format!(
            "celis: {} has not started; keep one step ahead: take it first, or drop it.\n",
            &first.id[..8]
        )
    );
}
