//! CIP-03, "Dropping".

mod support;

use support::{add, cli, err, finish, objects, out, refusal, Repo};

/// `drop` closes the step and every open step under it, and a reader leaves them out of `read`
/// (`spec/trails/03-steps.md`, "Dropping").
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
    let read = out(&cli(&repo, &["past"]));
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
/// gives it no grade (`spec/trails/04-trails.md`, "The commands").
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
    assert!(
        lines[0].starts_with(&format!("✗ {} the trail  dropped  ", &trail[..8])),
        "{text}"
    );
    assert_eq!(lines[1], "   dropped left behind", "{text}");
    assert!(
        lines[2].starts_with(&format!("  └─ ✗ {}", &part[..8])),
        "{text}"
    );
    assert_eq!(lines[3], "        dropped wrong step", "{text}");
    assert_eq!(lines.len(), 4, "{text}");
}

/// A drop with no reason is a usage error, and `drop` refuses an unknown step with `no-step` and a closed
/// one with `closed` (`spec/trails/03-steps.md`, "The commands").
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

/// `add --replaces` writes a step in the old step's place that names it, and drops the old step and the
/// open steps under it as replaced; a trail is not replaced (`spec/trails/03-steps.md`, "Dropping").
#[test]
fn a_step_that_replaces_another_drops_it() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let old = add(&repo, &["--under", &trail[..8], "--", "old plan"]);
    let deeper = add(&repo, &["--under", &old[..8], "--", "deeper part"]);
    let o = cli(
        &repo,
        &[
            "step",
            "add",
            "--replaces",
            &old[..8],
            "--",
            "new plan\nThe old plan missed the cache.",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let printed = out(&o);
    let written: Vec<&str> = printed.lines().collect();
    assert_eq!(written.len(), 3, "{printed}");
    let new = written[0].to_string();
    let tree = objects(&repo, &["tree", &trail[..8]]);
    let step = |id: &str| {
        tree.iter()
            .filter_map(|v| v.get("step"))
            .find(|s| s["id"] == id)
            .cloned()
            .unwrap()
    };
    let links = step(&new)["links"].clone();
    assert!(
        links
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l["kind"] == "under" && l["to"] == trail),
        "{links}"
    );
    assert!(
        links
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l["kind"] == "replaces" && l["to"] == old),
        "{links}"
    );
    let replaced = format!("replaced by {}", &new[..8]);
    for id in [&old, &deeper] {
        assert_eq!(step(id)["state"], "dropped");
        assert_eq!(step(id)["reason"], replaced.as_str());
    }
    let o = cli(
        &repo,
        &["step", "add", "--replaces", &trail[..8], "--", "x"],
    );
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
}
