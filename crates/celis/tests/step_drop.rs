//! CIP-03, "Dropping".

mod support;

use support::{
    account, add, cli, err, finish, live_ids, mark_note, now, objects, out, refusal, step_note,
    Repo, DAY,
};

/// `drop` closes the step and every open step under it, and a reader leaves them out of `read`
/// (`spec/03-steps.md`, "Dropping").
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
    let read = out(&cli(&repo, &["grep", " dropped "]));
    for id in [&trail, &open, &deeper] {
        assert!(read.contains(&format!(" {} dropped", &id[..8])), "{read}");
    }
    let mut states: Vec<String> = objects(&repo, &["tree", &trail[..8]])
        .iter()
        .filter_map(|v| v.get("step").or(v.get("trail")))
        .map(|s| s["state"].as_str().unwrap_or("").to_string())
        .collect();
    states.sort();
    assert_eq!(states, ["done", "dropped", "dropped", "dropped"]);
}

/// A dropped trail has the state `dropped`; `tree` marks a dropped trail and step `✗`, with no state word,
/// with its reason under it, and gives it no grade.
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
    assert_eq!(lines[0], format!("✗ {} the trail", &trail[..8]), "{text}");
    assert_eq!(lines[1], "  0/1 step", "{text}");
    assert_eq!(lines[2], "  dropped  left behind", "{text}");
    assert_eq!(lines[3], format!("  ✗ {} part", &part[..8]), "{text}");
    assert_eq!(lines[4], "    dropped  wrong step", "{text}");
    assert_eq!(lines.len(), 5, "{text}");
}

/// A drop with no reason is a usage error, and `drop` refuses an unknown step with `no-step` and a closed
/// one with `closed` (`spec/02-cli.md`, "Step").
#[test]
fn drop_refuses_an_empty_reason_and_unknown_and_closed_steps() {
    let repo = Repo::new();
    let open = add(&repo, &["--", "open"]);
    let closed = add(&repo, &["--under", &open[..8], "--", "closed"]);
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
/// open steps under it as replaced; a trail is not replaced (`spec/03-steps.md`, "Dropping").
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

/// `drop` takes a done trail, keeps it done, and refuses it once it is dropped
/// (`spec/02-cli.md`, "Step"; `spec/03-steps.md`, "Dropping").
#[test]
fn drop_takes_a_done_trail_once() {
    let repo = Repo::new();
    let note = step_note(&repo, &account(), None, "the trail", now() - DAY);
    mark_note(
        &repo,
        &account(),
        "done",
        &note.id,
        "proof",
        now() - DAY + 1,
    );
    let trail = note.id;
    let o = cli(&repo, &["step", "drop", &trail[..8], "--", "stale"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o).lines().count(), 1, "{}", out(&o));
    let state = &objects(&repo, &["tree", &trail[..8]])[0]["trail"]["state"];
    assert_eq!(state, "done");
    refusal(
        &cli(&repo, &["step", "drop", &trail[..8], "--json", "--", "x"]),
        1,
        "closed",
    );
}

/// `drop` takes a live rule and ends it, and refuses a rule that is not live
/// (`spec/02-cli.md`, "Step"; `spec/05-rules.md`, "Rules").
#[test]
fn drop_takes_a_live_rule_once() {
    let repo = Repo::new();
    let decide = |text: &str| {
        let o = cli(&repo, &["decide", "--", text]);
        assert!(o.status.success(), "{}", err(&o));
        out(&o).trim().to_string()
    };
    let kept = decide("When a: do a, because a.");
    let rule = decide("When b: do b, because b.");
    let o = cli(&repo, &["step", "drop", &rule[..8], "--", "merged into a"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(
        live_ids(&repo, &repo.path(), "repo"),
        [kept[..8].to_string()]
    );
    refusal(
        &cli(&repo, &["step", "drop", &rule[..8], "--json", "--", "x"]),
        1,
        "closed",
    );
}
