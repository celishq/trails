//! CIP-03, "Starting".

mod support;

use serde_json::Value;
use support::{add, cli, err, finish, objects, out, refusal, Repo};

/// The state `tree --json` gives the step `id`.
fn state(repo: &Repo, trail: &str, id: &str) -> String {
    objects(repo, &["tree", &trail[..8]])
        .into_iter()
        .find_map(|v| Some(v.get("step").filter(|s| s["id"] == id)?["state"].clone()))
        .and_then(|s| s.as_str().map(str::to_string))
        .unwrap()
}

/// A step is open, then working once a `start` names it, then done; a `start` does not close it, and
/// `tree` marks a working step `◐` (`spec/03-steps.md`, "Starting").
#[test]
fn a_started_step_is_working_until_it_closes() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let step = add(&repo, &["--under", &trail[..8], "--", "the step"]);
    assert_eq!(state(&repo, &trail, &step), "open");
    let o = cli(&repo, &["step", "start", &step[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(state(&repo, &trail, &step), "working");
    let tree = out(&cli(&repo, &["tree", &trail[..8]]));
    assert!(
        tree.lines()
            .any(|l| l.starts_with(&format!("  ◐ {} the step", &step[..8]))),
        "{tree}"
    );
    finish(&repo, &step[..8]);
    assert_eq!(state(&repo, &trail, &step), "done");
}

/// `start` refuses an unknown step with `no-step` and a closed step with `closed`; on a working step it
/// writes nothing and prints the id of the first `start`; it prints the id it wrote, and `{"id"}` with
/// `--json` (`spec/02-cli.md`, "Step").
#[test]
fn start_refuses_unknown_and_closed_steps_and_starts_a_step_once() {
    let repo = Repo::new();
    let open = add(&repo, &["--", "open"]);
    let closed = add(&repo, &["--", "closed"]);
    finish(&repo, &closed[..8]);
    refusal(
        &cli(&repo, &["step", "start", "deadbeef", "--json"]),
        1,
        "no-step",
    );
    refusal(
        &cli(&repo, &["step", "start", &closed[..8], "--json"]),
        1,
        "closed",
    );
    let files = || {
        std::fs::read_dir(repo.store().join("shared"))
            .unwrap()
            .count()
    };
    let o = cli(&repo, &["step", "start", &open[..8], "--json"]);
    assert!(o.status.success(), "{}", err(&o));
    let first: Value = serde_json::from_str(out(&o).trim()).unwrap();
    assert_eq!(first["id"].as_str().map(str::len), Some(64), "{}", out(&o));
    let before = files();
    let o = cli(&repo, &["step", "start", &open[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o).trim(), first["id"].as_str().unwrap());
    assert_eq!(files(), before);
}

/// `start` prints no pattern at the step's places; `celis search` gives them.
#[test]
fn start_prints_no_pattern_at_the_steps_places() {
    let repo = Repo::new();
    let o = cli(
        &repo,
        &["pattern", "--path", "src/a.rs", "--", "a.rs holds cents"],
    );
    assert!(o.status.success(), "{}", err(&o));
    let trail = add(&repo, &["--", "the trail"]);
    let args = [
        "--under",
        &trail[..8],
        "--at",
        "src/a.rs:4",
        "--at",
        "docs/b.md",
        "--",
        "s",
    ];
    let step = add(&repo, &args);
    let o = cli(&repo, &["step", "start", &step[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(!out(&o).contains("a.rs holds cents"), "{}", out(&o));
}

/// `start` does not run the step's check, and says nothing on stderr.
#[test]
fn start_does_not_run_the_steps_check() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let id = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--check",
            "touch ran",
            "--",
            "make it",
        ],
    );
    let o = cli(&repo, &["step", "start", &id[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(err(&o), "");
    assert!(!repo.path().join("ran").exists());
}

/// `start` prints the live patterns of the step's trail, each pattern its ask uses, and the patterns of the
/// trail of each step its ask uses; not a pattern none of them names.
#[test]
fn start_prints_the_trails_patterns_and_the_patterns_its_ask_uses() {
    let repo = Repo::new();
    let decide = |args: &[&str]| {
        let o = cli(&repo, &[&["pattern"], args].concat());
        assert!(o.status.success(), "{}", err(&o));
        out(&o).lines().next().unwrap().to_string()
    };
    let past = add(&repo, &["--", "past work"]);
    decide(&[
        "--scope",
        &format!("trail:{}", &past[..8]),
        "--",
        "past keeps tabs",
    ]);
    let used = decide(&["--", "use the shared Badge"]);
    decide(&["--", "never deploy on friday"]);
    let trail = add(&repo, &["--", "the trail"]);
    decide(&[
        "--scope",
        &format!("trail:{}", &trail[..8]),
        "--",
        "this trail rule",
    ]);
    let ask = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--uses",
            &used[..8],
            "--uses",
            &past[..8],
            "--",
            "the ask",
        ],
    );
    let step = add(&repo, &["--under", &ask[..8], "--", "s"]);
    let o = cli(&repo, &["step", "start", &step[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&o);
    for said in ["this trail rule", "use the shared Badge", "past keeps tabs"] {
        assert!(text.contains(said), "{text}");
    }
    assert!(!text.contains("never deploy on friday"), "{text}");
}
