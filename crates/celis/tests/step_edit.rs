//! CIP-03, "Editing".

mod support;

use support::{add, cli, err, objects, out, refusal, Repo};

/// `edit` gives a step new text, which `read`, `tree` and the JSON then show; it refuses an unknown id
/// and empty text (`spec/03-steps.md`, "Editing").
#[test]
fn edit_gives_a_step_new_text() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "Make the badges orange"]);
    let o = cli(
        &repo,
        &[
            "step",
            "edit",
            &trail[..4],
            "--",
            "Orange badges and buttons\nBoth use the token.",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o).trim().len(), 64, "{}", out(&o));
    let read = out(&cli(&repo, &["grep", "-i", "badge"]));
    assert!(read.contains("Orange badges and buttons"), "{read}");
    assert!(!read.contains("Make the badges"), "{read}");
    let tree = objects(&repo, &["tree", &trail[..8]]);
    assert_eq!(
        tree[0]["trail"]["content"],
        "Orange badges and buttons\nBoth use the token."
    );
    refusal(
        &cli(&repo, &["step", "edit", "deadbeef", "--json", "--", "x"]),
        1,
        "no-step",
    );
    refusal(
        &cli(&repo, &["step", "edit", &trail[..8], "--json", "--", " "]),
        2,
        "usage",
    );
}

/// `edit --check` and `--at` give a step a new check and places, and keep its text; a one-line text
/// changes only the title and keeps the detail (`spec/03-steps.md`, "Editing").
#[test]
fn edit_changes_the_check_and_places_and_a_title_keeps_the_detail() {
    let repo = Repo::new();
    let args = [
        "--at",
        "a.rs",
        "--check",
        "false",
        "--",
        "Old title\nThe detail.",
    ];
    let step = add(&repo, &args);
    let edit = |args: &[&str]| {
        let mut all = vec!["step", "edit", &step[..8]];
        all.extend(args);
        let o = cli(&repo, &all);
        assert!(o.status.success(), "{}", err(&o));
    };
    edit(&["--check", "true", "--at", "b.rs", "--at", "c.rs"]);
    edit(&["--", "New title"]);
    let tree = objects(&repo, &["tree", &step[..8]]);
    assert_eq!(tree[0]["trail"]["content"], "New title\nThe detail.");
    assert_eq!(
        tree[0]["trail"]["places"],
        serde_json::json!(["b.rs", "c.rs"])
    );
    let o = cli(
        &repo,
        &[
            "step",
            "done",
            &step[..8],
            "--as-planned",
            "--",
            "it passes",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    refusal(
        &cli(&repo, &["step", "edit", &step[..8], "--json"]),
        2,
        "usage",
    );
}

/// `step add` says on stderr when a session's step joins its trail (`spec/02-cli.md`,
/// "Step").
#[test]
fn add_says_when_a_step_joins_the_sessions_trail() {
    let repo = Repo::new();
    let env = [("CLAUDE_CODE_SESSION_ID", "s1")];
    let o = repo.run(&repo.path(), &["step", "add", "--", "the trail"], &env, "");
    assert!(!err(&o).contains("joined"), "{}", err(&o));
    let trail = out(&o).lines().nth(1).unwrap_or("").to_string();
    let o = repo.run(
        &repo.path(),
        &["step", "add", "--", "a later ask"],
        &env,
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(
        err(&o).trim(),
        format!(
            "celis: the step joined the trail {}; edit its title when it no longer covers the work.",
            &trail[..8]
        )
    );
}
