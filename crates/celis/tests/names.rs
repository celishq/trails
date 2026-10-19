//! CIP-07, "Names", and CIP-02, "Name".

mod support;

use support::{add, cli, err, out, refusal, Repo};

/// `name` writes a name on a trail and prints its id; the same name on the same trail again writes
/// nothing and prints that id (`spec/02-cli.md`, "Name").
#[test]
fn name_writes_a_name_once() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "Review a pull request"]);
    let first = cli(&repo, &["name", &trail[..8], "review"]);
    assert!(first.status.success(), "{}", err(&first));
    let id = out(&first).trim().to_string();
    assert_eq!(id.len(), 64, "{id}");
    let again = cli(&repo, &["name", &trail[..8], "review"]);
    assert!(again.status.success(), "{}", err(&again));
    assert_eq!(out(&again).trim(), id);
}

/// `name` refuses a step that is not a trail, a text that is no name, and a name another trail holds
/// (`spec/02-cli.md`, "Name"; `spec/07-names.md`, "Names").
#[test]
fn name_refuses_a_step_a_bad_name_and_a_held_name() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "Review a pull request"]);
    let step = add(&repo, &["--under", &trail, "--", "Read the diff"]);
    let other = add(&repo, &["--new", "--", "Review a design"]);
    refusal(
        &cli(&repo, &["name", &step[..8], "diff", "--json"]),
        2,
        "usage",
    );
    for bad in ["Review", "9review", "re view", "re_view", ""] {
        refusal(
            &cli(&repo, &["name", &trail[..8], bad, "--json"]),
            2,
            "usage",
        );
    }
    let o = cli(&repo, &["name", &trail[..8], "review"]);
    assert!(o.status.success(), "{}", err(&o));
    refusal(
        &cli(&repo, &["name", &other[..8], "review", "--json"]),
        2,
        "usage",
    );
}
