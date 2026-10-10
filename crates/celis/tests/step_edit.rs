//! CIP-12, "Editing".

mod support;

use support::{add, cli, err, objects, out, refusal, Repo};

/// `edit` gives a step new text, which `read`, `tree` and the JSON then show; it refuses an unknown id
/// and empty text (`spec/trails/12-steps.md`, "Editing").
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
    let read = out(&cli(&repo, &["read"]));
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

/// `step add` says on stderr when a session's step joins its trail (`spec/trails/12-steps.md`,
/// "Editing").
#[test]
fn add_says_when_a_step_joins_the_sessions_trail() {
    let repo = Repo::new();
    let env = [("CLAUDE_CODE_SESSION_ID", "s1")];
    let o = repo.run(&repo.path(), &["step", "add", "--", "the trail"], &env, "");
    assert!(err(&o).is_empty(), "{}", err(&o));
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
