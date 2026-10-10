//! CIP-12, "Dropping".

mod support;

use support::{add, cli, err, finish, objects, out, refusal, Repo};

/// `drop` closes the step and every open step under it, and a reader leaves them out of `read`
/// (`spec/trails/12-steps.md`, "Dropping").
#[test]
fn drop_closes_the_step_and_the_open_steps_under_it() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let done = add(&repo, &["--under", &trail[..8], "--", "done part"]);
    let open = add(&repo, &["--under", &trail[..8], "--", "open part"]);
    let deeper = add(&repo, &["--under", &open[..8], "--", "deeper part"]);
    finish(&repo, &done[..8]);
    let o = cli(
        &repo,
        &["step", "drop", &trail[..8], "--", "no longer wanted"],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o).lines().count(), 3, "{}", out(&o));
    let read = out(&cli(&repo, &["read"]));
    for id in [&trail, &open, &deeper] {
        assert!(!read.contains(&id[..8]), "{read}");
    }
    let mut states: Vec<String> = objects(&repo, &["tree", &trail[..8]])
        .iter()
        .filter_map(|v| v.get("step").or(v.get("trail")))
        .map(|s| s["state"].as_str().unwrap_or("").to_string())
        .collect();
    states.sort();
    assert_eq!(states, ["done", "dropped", "dropped", "dropped"]);
}

/// A dropped trail has the state `dropped`; `tree` marks a dropped step `✗` with its reason under it, and
/// gives it no grade (`spec/trails/14-trails.md`, "The commands").
#[test]
fn tree_marks_a_dropped_step_with_its_reason_and_no_grade() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "part"]);
    let o = cli(&repo, &["step", "drop", &part[..8], "--", "wrong step"]);
    assert!(o.status.success(), "{}", err(&o));
    let o = cli(&repo, &["step", "drop", &trail[..8], "--", "left behind"]);
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&cli(&repo, &["tree", &trail[..8], "--full"]));
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[0].ends_with(" · dropped"), "{text}");
    assert_eq!(lines[1], "    dropped left behind", "{text}");
    assert!(lines[2].starts_with(&format!("✗ {}", &part[..8])), "{text}");
    assert_eq!(lines[3], "    dropped wrong step", "{text}");
    assert_eq!(lines.len(), 4, "{text}");
}

/// A drop with no reason is a usage error, and `drop` refuses an unknown step with `no-step` and a closed
/// one with `closed` (`spec/trails/12-steps.md`, "The commands").
#[test]
fn drop_refuses_an_empty_reason_and_unknown_and_closed_steps() {
    let repo = Repo::new();
    let open = add(&repo, &["--", "open"]);
    let closed = add(&repo, &["--", "closed"]);
    finish(&repo, &closed[..8]);
    let o = cli(&repo, &["step", "drop", &open[..8], "--", " "]);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    refusal(
        &cli(&repo, &["step", "drop", "deadbeef", "--json", "--", "x"]),
        1,
        "no-step",
    );
    refusal(
        &cli(&repo, &["step", "drop", &closed[..8], "--json", "--", "x"]),
        1,
        "closed",
    );
    let o = cli(&repo, &["step", "start", &open[..8], "--json"]);
    assert!(o.status.success(), "{}", err(&o));
    let o = cli(&repo, &["step", "drop", &open[..8], "--", "x"]);
    assert!(o.status.success(), "{}", err(&o));
    refusal(
        &cli(&repo, &["step", "start", &open[..8], "--json"]),
        1,
        "closed",
    );
}
