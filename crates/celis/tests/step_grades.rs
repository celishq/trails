//! CIP-12, "Grades".

mod support;

use serde_json::Value;
use std::process::Output;
use support::{err, out, refusal, Repo};

fn run(repo: &Repo, args: &[&str]) -> Output {
    repo.run(&repo.path(), args, &[], "")
}

/// The id `step add <args>` prints.
fn add(repo: &Repo, args: &[&str]) -> String {
    let mut all = vec!["step", "add"];
    all.extend(args);
    let o = run(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).trim().to_string()
}

fn done(repo: &Repo, id: &str, extra: &[&str]) -> Output {
    let mut all = vec!["step", "done", id];
    all.extend(extra);
    all.extend(["--", "failed before, passes now"]);
    run(repo, &all)
}

fn records(repo: &Repo) -> usize {
    std::fs::read_dir(repo.store().join("shared")).map_or(0, |d| d.count())
}

/// `done` runs the step's check, and refuses with `check-failed` and writes nothing when it fails
/// (`spec/trails/12-steps.md`, "The commands").
#[test]
fn done_runs_the_check_and_refuses_when_it_fails() {
    let repo = Repo::new();
    let failing = add(&repo, &["--check", "echo broken; false", "--", "fails"]);
    let before = records(&repo);
    let o = done(&repo, &failing[..8], &["--json"]);
    refusal(&o, 1, "check-failed");
    assert!(err(&o).contains("broken"), "{}", err(&o));
    assert_eq!(records(&repo), before);
    let passing = add(&repo, &["--check", "true", "--", "passes"]);
    let o = done(&repo, &passing[..8], &[]);
    assert!(o.status.success(), "{}", err(&o));
}

/// `tree` prints a done step's grade under its proof: the time it took, the steps that fix it as
/// rework, and its steers (`spec/trails/12-steps.md`, "Grades").
#[test]
fn tree_prints_the_grade_of_each_done_step() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let badge = add(&repo, &["--under", &trail[..8], "--", "badge"]);
    let o = run(
        &repo,
        &["step", "steer", &badge[..8], "--", "Use the shared Badge."],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert!(done(&repo, &badge[..8], &[]).status.success());
    add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--fixes",
            &badge[..8],
            "--",
            "the badge repeats",
        ],
    );
    let o = run(&repo, &["tree", &trail[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&o);
    let lines: Vec<&str> = text.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.contains(&badge[..8]))
        .expect("the badge");
    assert_eq!(
        lines[at + 1],
        "    done failed before, passes now",
        "{text}"
    );
    assert!(lines[at + 2].starts_with("    took "), "{text}");
    assert!(
        lines[at + 2].ends_with("s · rework 1 · steered 1"),
        "{text}"
    );
}

/// `tree --json` gives a done step its grade (`spec/trails/12-steps.md`, "Grades").
#[test]
fn tree_json_gives_a_done_step_its_grade() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let id = add(&repo, &["--under", &trail[..8], "--", "work"]);
    assert!(done(&repo, &id[..8], &[]).status.success());
    let o = run(&repo, &["tree", &trail[..8], "--json"]);
    assert!(o.status.success(), "{}", err(&o));
    let step: Value = out(&o)
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .find_map(|v| v.get("step").cloned())
        .expect("a step");
    let grade = step["grade"].as_object().expect("a grade");
    let keys: Vec<&str> = grade.keys().map(String::as_str).collect();
    assert_eq!(keys, ["rework", "steered", "took"]);
}
