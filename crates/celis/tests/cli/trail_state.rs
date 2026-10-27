//! CIP-04, "State".

use crate::support;

use support::{add, cli, err, finish, objects, out, Repo};

/// The state `tree <id> --json` gives the trail `id`.
fn state(repo: &Repo, id: &str) -> String {
    objects(repo, &["tree", &id[..8]])[0]["trail"]["state"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

/// A trail is active while open, and reached once done; a reader ignores an old steer.
#[test]
fn a_trails_state_comes_from_its_tree_and_ignores_steers() {
    let repo = Repo::new();
    let alone = add(&repo, &["--", "alone"]);
    let part = add(&repo, &["--under", &alone[..8], "--", "part"]);
    assert_eq!(state(&repo, &alone), "open");
    finish(&repo, &part[..8]);
    finish(&repo, &alone[..8]);
    assert_eq!(state(&repo, &alone), "done");

    let helped = add(&repo, &["--", "helped"]);
    let part = add(&repo, &["--under", &helped[..8], "--", "part two"]);
    let deeper = add(&repo, &["--under", &part[..8], "--", "deeper"]);
    support::mark_note(
        &repo,
        &support::account(),
        "steer",
        &deeper,
        "Use a token.",
        support::now(),
    );
    finish(&repo, &deeper[..8]);
    finish(&repo, &part[..8]);
    finish(&repo, &helped[..8]);
    assert_eq!(state(&repo, &helped), "done");
    let tree = out(&cli(&repo, &["tree", &helped[..8], "--full"]));
    assert!(!tree.contains("steered"), "{tree}");
}

/// A trail with no open step under it stays open until its own `done`, and a done trail refuses a new
/// step under it (`spec/04-trails.md`, "State").
#[test]
fn a_trail_takes_the_state_of_its_ask() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "part"]);
    finish(&repo, &part[..8]);
    assert_eq!(state(&repo, &trail), "open");
    finish(&repo, &trail[..8]);
    assert_eq!(state(&repo, &trail), "done");
    let read = out(&cli(&repo, &["grep", " trail "]));
    assert!(
        read.lines()
            .any(|l| l == format!("  trail {} done", &trail[..8])),
        "{read}"
    );
    let locked = add(&repo, &["--", "locked"]);
    finish(&repo, &locked[..8]);
    let o = cli(
        &repo,
        &["step", "add", "--under", &locked[..8], "--", "late"],
    );
    assert!(!o.status.success(), "{}", out(&o));
}

/// A trail that ended 30 minutes after the newest record of its tree stays open with no `done` of its
/// own, and shows its open steps stale (`spec/04-trails.md`, "State"; `spec/02-cli.md`, "Grep").
#[test]
fn an_ended_trail_stays_open_and_shows_its_open_steps_stale() {
    use support::{account, mark_note, now, step_note};
    let repo = Repo::new();
    let old = now() - 31 * 60;
    let finished = step_note(&repo, &account(), None, "finished", old);
    let part = step_note(&repo, &account(), Some(&finished), "part", old);
    mark_note(&repo, &account(), "done", &part.id, "proof", old);
    let left = step_note(&repo, &account(), None, "left", old);
    let open = step_note(&repo, &account(), Some(&left), "open part", old);
    assert_eq!(state(&repo, &finished.id), "open");
    assert_eq!(state(&repo, &left.id), "open");
    let tree = out(&cli(&repo, &["tree", &left.id[..8]]));
    assert!(
        tree.lines()
            .nth(2)
            .unwrap_or("")
            .starts_with(&format!("  ◌   {} open part", &open.id[..8])),
        "{tree}"
    );
    let read = out(&cli(&repo, &["grep", "left\\|open part"]));
    assert!(
        read.lines()
            .any(|l| l == format!("  trail {} open", &left.id[..8])),
        "{read}"
    );
    assert!(
        read.contains(&format!(" step {} open open part", &open.id[..8])),
        "{read}"
    );
    let o = cli(&repo, &["step", "start", &open.id[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(state(&repo, &left.id), "open");
}
