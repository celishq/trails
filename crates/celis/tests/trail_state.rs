//! CIP-04, "State".

mod support;

use support::{add, cli, err, finish, objects, out, Repo};

/// The state `tree <id> --json` gives the trail `id`.
fn state(repo: &Repo, id: &str) -> String {
    objects(repo, &["tree", &id[..8]])[0]["trail"]["state"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

/// A trail is active while open, and reached once done; a reader ignores an old steer
/// (`spec/trails/04-trails.md`, "State").
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
    support::steer(&repo, &deeper, "Use a token.");
    finish(&repo, &deeper[..8]);
    finish(&repo, &part[..8]);
    finish(&repo, &helped[..8]);
    assert_eq!(state(&repo, &helped), "done");
    let tree = out(&cli(&repo, &["tree", &helped[..8], "--full"]));
    assert!(!tree.contains("steered"), "{tree}");
}

/// A trail with steps under it and none open is reached at once; a new step under it makes it active
/// again, and a done trail refuses one (`spec/trails/04-trails.md`, "State").
#[test]
fn a_trail_with_no_open_step_under_it_is_reached_at_once() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "part"]);
    finish(&repo, &part[..8]);
    assert_eq!(state(&repo, &trail), "done");
    let read = out(&cli(&repo, &["past"]));
    let active = format!("{} [repo] the trail", &trail[..8]);
    assert!(!read.lines().any(|l| l == active), "{read}");
    add(&repo, &["--under", &trail[..8], "--", "more"]);
    assert_eq!(state(&repo, &trail), "open");
    let locked = add(&repo, &["--", "locked"]);
    finish(&repo, &locked[..8]);
    let o = cli(
        &repo,
        &["step", "add", "--under", &locked[..8], "--", "late"],
    );
    assert!(!o.status.success(), "{}", out(&o));
}

/// A trail ends 30 minutes after the newest record of its tree: reached when nothing under it is open,
/// expired when a step is, and active again when a new record lands; an ended trail's steps are not open
/// (`spec/trails/04-trails.md`, "State").
#[test]
fn a_trail_ends_thirty_minutes_after_its_newest_record() {
    use support::{account, done_note, now, step_note};
    let repo = Repo::new();
    let old = now() - 31 * 60;
    let finished = step_note(&repo, &account(), None, "finished", old);
    let part = step_note(&repo, &account(), Some(&finished), "part", old);
    done_note(&repo, &account(), &part, old);
    let left = step_note(&repo, &account(), None, "left", old);
    let open = step_note(&repo, &account(), Some(&left), "open part", old);
    assert_eq!(state(&repo, &finished.id), "done");
    assert_eq!(state(&repo, &left.id), "expired");
    let tree = out(&cli(&repo, &["tree", &left.id[..8]]));
    assert!(
        tree.lines().nth(1).unwrap_or("").starts_with("  └─ ! "),
        "{tree}"
    );
    let read = out(&cli(&repo, &["past"]));
    let active = format!("{} [repo] left", &left.id[..8]);
    assert!(!read.lines().any(|l| l == active), "{read}");
    assert!(!read.contains(&open.id[..8]), "{read}");
    let o = cli(&repo, &["step", "start", &open.id[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(state(&repo, &left.id), "working");
}

/// A trail whose session wrote no end ends 30 minutes after the newest record of its tree
/// (`spec/trails/04-trails.md`, "State").
#[test]
fn a_sessions_trail_with_no_end_ends_thirty_minutes_after_its_newest_record() {
    use support::{account, now, put, step_tags};
    let repo = Repo::new();
    let trail = |text: &str, at: u64| {
        let mut tags = step_tags((&repo.project_id(), "repo"));
        tags.push(vec!["session".to_string(), "s1".to_string()]);
        let note = celis_nostr::Note::write(&account(), 1111u16, at, tags, text).unwrap();
        put(&repo, &note);
        note
    };
    let recent = trail("ten minutes ago", now() - 10 * 60);
    let quiet = trail("an hour ago", now() - 60 * 60);
    assert_eq!(state(&repo, &recent.id), "open");
    assert_eq!(state(&repo, &quiet.id), "done");
}
