//! CIP-03, "Grades".

mod support;

use celis_nostr::Note;
use serde_json::Value;
use std::process::Output;
use support::{account, done_note, err, now, out, put, refusal, step_note, Repo};

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
    let mut all = vec!["step", "done", id, "--as-planned"];
    all.extend(extra);
    all.extend(["--", "failed before, passes now"]);
    run(repo, &all)
}

fn records(repo: &Repo) -> usize {
    std::fs::read_dir(repo.store().join("shared")).map_or(0, |d| d.count())
}

/// `done` runs the step's check, and refuses with `check-failed` and writes nothing when it fails
/// (`spec/02-cli.md`, "Step").
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

/// `done` runs the check in the project's top folder, from any folder under it, and stops a check that
/// runs past `CELIS_CHECK_SECONDS` as failed (`spec/02-cli.md`, "Step").
#[test]
fn done_runs_the_check_at_the_top_folder_and_stops_a_slow_one() {
    let repo = Repo::new();
    repo.write("marker", "");
    repo.write("sub/keep", "");
    let top = add(&repo, &["--check", "test -f marker", "--", "at the top"]);
    let o = repo.run(
        &repo.path().join("sub"),
        &["step", "done", &top[..8], "--as-planned", "--", "p"],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    let slow = add(&repo, &["--check", "sleep 30", "--", "slow"]);
    let started = std::time::Instant::now();
    let o = repo.run(
        &repo.path(),
        &[
            "step",
            "done",
            &slow[..8],
            "--json",
            "--as-planned",
            "--",
            "p",
        ],
        &[("CELIS_CHECK_SECONDS", "1")],
        "",
    );
    refusal(&o, 1, "check-failed");
    assert!(started.elapsed().as_secs() < 10, "{:?}", started.elapsed());
}

/// `tree` prints a done step's grade under its proof: the time it took
/// (`spec/02-cli.md`, "Tree").
#[test]
fn tree_prints_the_grade_of_each_done_step() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let badge = add(&repo, &["--under", &trail[..8], "--", "badge"]);
    assert!(done(&repo, &badge[..8], &[]).status.success());
    let o = run(&repo, &["tree", &trail[..8], "--full"]);
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&o);
    let lines: Vec<&str> = text.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.starts_with("  ✓ badge") && l.ends_with(&badge[..8]))
        .expect("the badge");
    let under = |n: usize| lines[at + n].trim_start_matches([' ', '│']);
    assert_eq!(under(1), "done    failed before, passes now", "{text}");
    assert!(under(2).starts_with("took    "), "{text}");
    assert!(under(2).ends_with('s'), "{text}");
}

/// `tree --json` gives a done step its grade (`spec/02-cli.md`, "Tree").
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
    assert_eq!(keys, ["revised", "took"]);
}

/// `took` counts from the step's first `start`, and from the step when it has none
/// (`spec/03-steps.md`, "Grades").
#[test]
fn took_counts_from_the_first_start_else_from_the_step() {
    let repo = Repo::new();
    let at = now() - 10_000;
    let trail = step_note(&repo, &account(), None, "the trail", at);
    let started = step_note(&repo, &account(), Some(&trail), "started", at);
    let tags: Vec<Vec<String>> = vec![
        vec!["L".into(), "celis.step".into()],
        vec!["l".into(), "start".into(), "celis.step".into()],
        vec!["e".into(), started.id.clone()],
    ];
    put(
        &repo,
        &Note::write(&account(), 1985u16, at + 3_000, tags, "").unwrap(),
    );
    done_note(&repo, &account(), &started, at + 3_090);
    let unstarted = step_note(&repo, &account(), Some(&trail), "unstarted", at);
    done_note(&repo, &account(), &unstarted, at + 90);
    let o = run(&repo, &["tree", &trail.id[..8], "--full"]);
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&o);
    let took = |id: &str| {
        let lines: Vec<&str> = text.lines().collect();
        let at = lines.iter().position(|l| l.contains(id)).expect("the step");
        lines[at + 2].trim_start_matches([' ', '│']).to_string()
    };
    assert_eq!(took(&started.id[..8]), "took    1m 30s", "{text}");
    assert_eq!(took(&unstarted.id[..8]), "took    1m 30s", "{text}");
}
