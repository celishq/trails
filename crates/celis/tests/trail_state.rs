//! CIP-14, "State".

mod support;

use support::{add, cli, err, finish, out, Repo};

/// The first line `tree <id>` prints.
fn head(repo: &Repo, id: &str) -> String {
    let o = cli(repo, &["tree", &id[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().next().unwrap_or("").to_string()
}

/// A trail is active while open, reached alone when done with no steer in its tree, and reached with
/// help when a steer names a step of its tree (`spec/trails/14-trails.md`, "State").
#[test]
fn a_trails_state_comes_from_its_tree_and_its_steers() {
    let repo = Repo::new();
    let alone = add(&repo, &["--", "alone"]);
    let part = add(&repo, &["--under", &alone[..8], "--", "part"]);
    assert!(head(&repo, &alone).ends_with(" · active"));
    finish(&repo, &part[..8]);
    assert!(head(&repo, &alone).ends_with(" · active"));
    finish(&repo, &alone[..8]);
    assert!(head(&repo, &alone).ends_with(" · reached alone"));

    let helped = add(&repo, &["--", "helped"]);
    let part = add(&repo, &["--under", &helped[..8], "--", "part two"]);
    let deeper = add(&repo, &["--under", &part[..8], "--", "deeper"]);
    let o = cli(
        &repo,
        &["step", "steer", &deeper[..8], "--", "Use a token."],
    );
    assert!(o.status.success(), "{}", err(&o));
    finish(&repo, &deeper[..8]);
    finish(&repo, &part[..8]);
    finish(&repo, &helped[..8]);
    assert!(head(&repo, &helped).ends_with(" · reached with help"));
}
