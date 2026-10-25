//! CIP-02, "Commands".

mod support;

use serde_json::Value;
use std::process::Output;
use support::{
    account, err, labelled, mark_note, now, out, refusal, step_note, stored_pattern, Repo,
};

const WORK: &str = "When you change prices: keep them in cents, because floats lost a cent.";

fn run(repo: &Repo, args: &[&str]) -> Output {
    repo.run(&repo.path(), args, &[], "")
}

fn short(id: &str) -> String {
    id[..8].to_string()
}

fn records(repo: &Repo) -> usize {
    std::fs::read_dir(repo.store().join("shared"))
        .map(|d| d.count())
        .unwrap_or(0)
}

/// The lines `celis <args>` prints in `folder`.
fn lines_of(repo: &Repo, folder: &std::path::Path, args: &[&str]) -> Vec<String> {
    let o = repo.run(folder, args, &[], "");
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().map(str::to_string).collect()
}

fn project(id: &str) -> (&str, &str) {
    (id, "repo")
}

/// A pattern prints as `<id, first 8> <scope> <text>`.
#[test]
fn a_pattern_prints_its_id_scope_and_text() {
    let repo = Repo::new();
    let o = repo.run(
        &repo.path(),
        &["decide", "--scope", "me", "--", "one"],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    let placed = out(&o).trim().to_string();
    let step = run(&repo, &["step", "add", "--uses", &placed, "--", "t"]);
    let step = short(out(&step).trim());
    let lines = lines_of(&repo, &repo.path(), &["step", "start", &step]);
    assert_eq!(lines[1..], [format!("{} me one", short(&placed))]);
}

/// The json form of a pattern is one object with the fields the section lists
/// (`spec/02-cli.md`, "Commands").
#[test]
fn the_json_form_of_a_pattern_has_the_listed_fields() {
    let repo = Repo::new();
    let id = repo.project_id();
    let note = stored_pattern(
        &repo,
        project(&id),
        &account(),
        (Some("at:a.py"), None),
        "one",
        now() - 60,
    );
    let step = run(&repo, &["step", "add", "--uses", &note.id, "--", "t"]);
    let step = short(out(&step).trim());
    let o = repo.run(&repo.path(), &["step", "start", &step, "--json"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let line: Value = serde_json::from_str(out(&o).lines().nth(1).unwrap()).unwrap();
    let pattern = line["decision"].as_object().unwrap();
    let mut keys: Vec<&str> = pattern.keys().map(String::as_str).collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "binds",
            "content",
            "created_at",
            "id",
            "paths",
            "project",
            "promoted",
            "replaces",
            "scope"
        ]
    );
    assert_eq!(pattern["id"], note.id.as_str());
    assert_eq!(pattern["scope"], "at:a.py");
}

/// `decide` writes a pattern on the working folder's project with `--scope` and `--replaces` as its
/// tags, the whole id of a trail, and no `scope` tag for the project (`spec/02-cli.md`,
/// "Commands").
#[test]
fn decide_writes_its_scope_and_replaces_as_tags() {
    let repo = Repo::new();
    let id = repo.project_id();
    let trail = out(&run(&repo, &["step", "add", "--", "t"]))
        .trim()
        .to_string();
    let old = stored_pattern(
        &repo,
        project(&id),
        &account(),
        (None, None),
        "old",
        now() - 60,
    );
    let written = |args: &[&str]| {
        let mut all = vec!["decide"];
        all.extend(args);
        all.extend(["--", WORK]);
        let o = repo.run(&repo.path(), &all, &[], "");
        assert!(o.status.success(), "{}", err(&o));
        let id = out(&o).trim().to_string();
        labelled(&repo, "decision")
            .into_iter()
            .find(|n| n.id == id)
            .unwrap()
    };
    let replacing = written(&[
        "--scope",
        "at:shelf/prices.py",
        "--replaces",
        &short(&old.id),
    ]);
    assert_eq!(replacing.content, WORK);
    assert_eq!(replacing.tag("scope"), None);
    assert_eq!(replacing.tag("path"), Some("shelf/prices.py"));
    assert_eq!(replacing.tag("replaces"), Some(old.id.as_str()));
    assert_eq!(replacing.tag("I"), Some(format!("project:{id}").as_str()));
    let pattern = written(&["--scope", &format!("trail:{}", short(&trail))]);
    assert_eq!(
        pattern.tag("scope"),
        Some(format!("trail:{trail}").as_str())
    );
    assert_eq!(written(&["--scope", "project"]).tag("scope"), None);
}

/// `decide` refuses empty text, and a scope it does not know, as a usage error
/// (`spec/02-cli.md`, "Commands").
#[test]
fn decide_refuses_empty_text_and_an_unknown_scope_as_a_usage_error() {
    let repo = Repo::new();
    for args in [
        vec!["decide", "--json", "--", ""],
        vec!["decide", "--json"],
        vec!["decide", "--json", "--scope", "team:design", "--", WORK],
        vec!["decide", "--json", "--scope", "at:", "--", WORK],
    ] {
        refusal(&repo.run(&repo.path(), &args, &[], ""), 2, "usage");
    }
    assert_eq!(records(&repo), 0);
}

/// `decide` refuses `trail:<id>` when no trail of the project that is not done or dropped has the id,
/// with `no-trail`; it takes a reached one (`spec/02-cli.md`, "Commands").
#[test]
fn decide_refuses_a_trail_the_project_has_not_open() {
    let repo = Repo::new();
    let t = now() - 5 * 60;
    let done = step_note(&repo, &account(), None, "reached", t);
    mark_note(&repo, &account(), "done", &done.id, "proof", t + 1);
    let active = step_note(&repo, &account(), None, "active", t + 2);
    let part = step_note(&repo, &account(), Some(&active), "part", t + 3);
    let unknown = "0".repeat(64);
    for trail in [unknown.as_str(), done.id.as_str(), part.id.as_str()] {
        let scope = format!("trail:{trail}");
        let args = ["decide", "--json", "--scope", &scope, "--", WORK];
        refusal(&repo.run(&repo.path(), &args, &[], ""), 1, "no-trail");
    }
    let shop = repo.other("shop");
    let scope = format!("trail:{}", active.id);
    let args = ["decide", "--json", "--scope", &scope, "--", WORK];
    refusal(&repo.run(&shop, &args, &[], ""), 1, "no-trail");
    assert!(labelled(&repo, "decision").is_empty());
    mark_note(&repo, &account(), "done", &part.id, "proof", t + 4);
    let args = ["decide", "--scope", &scope, "--", WORK];
    let o = repo.run(&repo.path(), &args, &[], "");
    assert!(o.status.success(), "{}", err(&o));
}

/// `decide` takes `trail:<id>` of a done trail when `--replaces` names a live pattern of that trail
/// (`spec/02-cli.md`, "Commands").
#[test]
fn decide_keeps_a_done_trails_scope_when_it_replaces_its_pattern() {
    let repo = Repo::new();
    let t = now() - 5 * 60;
    let trail = step_note(&repo, &account(), None, "reached", t);
    let scope = format!("trail:{}", trail.id);
    let first = repo.run(
        &repo.path(),
        &["decide", "--scope", &scope, "--", WORK],
        &[],
        "",
    );
    assert!(first.status.success(), "{}", err(&first));
    let old = String::from_utf8_lossy(&first.stdout).trim().to_string();
    mark_note(&repo, &account(), "done", &trail.id, "proof", t + 1);
    let args = ["decide", "--json", "--scope", &scope, "--", WORK];
    refusal(&repo.run(&repo.path(), &args, &[], ""), 1, "no-trail");
    let args = [
        "decide",
        "--scope",
        &scope,
        "--replaces",
        &old,
        "--path",
        "**/*.py",
        "--",
        WORK,
    ];
    let o = repo.run(&repo.path(), &args, &[], "");
    assert!(o.status.success(), "{}", err(&o));
}

/// `decide` refuses a `--replaces` no live pattern has, with `no-decision`
/// (`spec/02-cli.md`, "Commands").
#[test]
fn decide_refuses_a_replaces_no_live_pattern_has() {
    let repo = Repo::new();
    let id = repo.project_id();
    let old = stored_pattern(
        &repo,
        project(&id),
        &account(),
        (None, None),
        "old",
        now() - 3 * 60,
    );
    stored_pattern(
        &repo,
        project(&id),
        &account(),
        (None, Some(&old.id)),
        "new",
        now() - 2 * 60,
    );
    let unknown = "0".repeat(64);
    for target in [unknown.as_str(), old.id.as_str()] {
        let args = ["decide", "--json", "--replaces", target, "--", WORK];
        refusal(&repo.run(&repo.path(), &args, &[], ""), 1, "no-decision");
    }
    assert_eq!(labelled(&repo, "decision").len(), 2);
}

/// A flag no command takes is a usage error (`spec/02-cli.md`, "Commands").
#[test]
fn a_flag_no_command_takes_is_a_usage_error() {
    let repo = Repo::new();
    for args in [
        vec!["tree", "--json", "--bogus"],
        vec!["decide", "--json", "--bogus", "--", WORK],
        vec!["step", "add", "--json", "--needs", "x", "--", "work"],
    ] {
        refusal(&repo.run(&repo.path(), &args, &[], ""), 2, "usage");
    }
}
