//! CIP-02, "Commands".

mod support;

use serde_json::Value;
use std::process::Output;
use support::{account, err, labelled, now, out, refusal, stored_pattern, Repo};

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
        &["pattern", "--scope", "me", "--", "one"],
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
        (Some("me"), None),
        "one",
        now() - 60,
    );
    let step = run(&repo, &["step", "add", "--uses", &note.id, "--", "t"]);
    let step = short(out(&step).trim());
    let o = repo.run(&repo.path(), &["step", "start", &step, "--json"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let line: Value = serde_json::from_str(out(&o).lines().nth(1).unwrap()).unwrap();
    let pattern = line["pattern"].as_object().unwrap();
    let mut keys: Vec<&str> = pattern.keys().map(String::as_str).collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "binds",
            "cells",
            "content",
            "created_at",
            "id",
            "paths",
            "project",
            "replaces",
            "scope"
        ]
    );
    assert_eq!(pattern["id"], note.id.as_str());
    assert_eq!(pattern["scope"], "me");
}

/// `pattern` writes a pattern on the working folder's project with `--scope` and `--replaces` as its
/// tags, and no `scope` tag for the project (`spec/02-cli.md`, "Commands").
#[test]
fn pattern_writes_its_scope_and_replaces_as_tags() {
    let repo = Repo::new();
    let id = repo.project_id();
    let old = stored_pattern(
        &repo,
        project(&id),
        &account(),
        (None, None),
        "old",
        now() - 60,
    );
    let written = |args: &[&str]| {
        let mut all = vec!["pattern"];
        all.extend(args);
        all.extend(["--", WORK]);
        let o = repo.run(&repo.path(), &all, &[], "");
        assert!(o.status.success(), "{}", err(&o));
        let id = out(&o).lines().next().unwrap().to_string();
        labelled(&repo, "pattern")
            .into_iter()
            .find(|n| n.id == id)
            .unwrap()
    };
    let replacing = written(&["--path", "shelf/prices.py", "--replaces", &short(&old.id)]);
    assert_eq!(replacing.content, WORK);
    assert_eq!(replacing.tag("scope"), None);
    assert_eq!(replacing.tag("path"), Some("shelf/prices.py"));
    assert_eq!(replacing.tag("replaces"), Some(old.id.as_str()));
    assert_eq!(replacing.tag("I"), Some(format!("project:{id}").as_str()));
    assert_eq!(written(&["--scope", "project"]).tag("scope"), None);
}

/// `pattern` refuses empty text, and a scope it does not know, as a usage error
/// (`spec/02-cli.md`, "Commands").
#[test]
fn pattern_refuses_empty_text_and_an_unknown_scope_as_a_usage_error() {
    let repo = Repo::new();
    for args in [
        vec!["pattern", "--json", "--", ""],
        vec!["pattern", "--json"],
        vec!["pattern", "--json", "--scope", "team:design", "--", WORK],
        vec!["pattern", "--json", "--scope", "at:a.py", "--", WORK],
    ] {
        refusal(&repo.run(&repo.path(), &args, &[], ""), 2, "usage");
    }
    assert_eq!(records(&repo), 0);
}

/// `pattern` refuses the scope `trail:<id>` as a usage error that names `step note`, and writes nothing
/// (`spec/02-cli.md`, "Commands").
#[test]
fn pattern_refuses_a_trail_scope_and_names_step_note() {
    let repo = Repo::new();
    let trail = out(&run(&repo, &["step", "add", "--", "t"]))
        .trim()
        .to_string();
    let scope = format!("trail:{trail}");
    let o = run(&repo, &["pattern", "--json", "--scope", &scope, "--", WORK]);
    refusal(&o, 2, "usage");
    assert!(err(&o).contains("step note"), "{}", err(&o));
    assert!(labelled(&repo, "pattern").is_empty());
}

/// `pattern` refuses a `--replaces` no live pattern has, with `no-pattern`
/// (`spec/02-cli.md`, "Commands").
#[test]
fn pattern_refuses_a_replaces_no_live_pattern_has() {
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
        let args = ["pattern", "--json", "--replaces", target, "--", WORK];
        refusal(&repo.run(&repo.path(), &args, &[], ""), 1, "no-pattern");
    }
    assert_eq!(labelled(&repo, "pattern").len(), 2);
}

/// A flag no command takes is a usage error (`spec/02-cli.md`, "Commands").
#[test]
fn a_flag_no_command_takes_is_a_usage_error() {
    let repo = Repo::new();
    for args in [
        vec!["tree", "--json", "--bogus"],
        vec!["pattern", "--json", "--bogus", "--", WORK],
        vec!["step", "add", "--json", "--needs", "x", "--", "work"],
    ] {
        refusal(&repo.run(&repo.path(), &args, &[], ""), 2, "usage");
    }
}

/// `decide` is no command: it is a usage error, and writes nothing (`spec/02-cli.md`, "Commands").
#[test]
fn decide_is_a_usage_error() {
    let repo = Repo::new();
    let o = repo.run(&repo.path(), &["decide", "--json", "--", WORK], &[], "");
    refusal(&o, 2, "usage");
    assert!(labelled(&repo, "pattern").is_empty());
    assert!(labelled(&repo, "decision").is_empty());
}
