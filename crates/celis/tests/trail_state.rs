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

/// A trail ends 30 minutes after the newest record of its tree: reached when nothing under it is open,
/// expired when a step is, and active again when a new record lands; an ended trail's steps are not open
/// (`spec/trails/14-trails.md`, "State").
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
    assert!(head(&repo, &finished.id).ends_with(" · reached alone"));
    assert!(head(&repo, &left.id).ends_with(" · expired"));
    let read = out(&cli(&repo, &["read"]));
    assert!(!read.contains(&left.id[..8]), "{read}");
    assert!(!read.contains(&open.id[..8]), "{read}");
    let o = cli(&repo, &["step", "start", &open.id[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(head(&repo, &left.id).ends_with(" · active"));
}

/// A trail with a session lives 12 hours after the newest record of its tree, not 30 minutes
/// (`spec/trails/14-trails.md`, "State").
#[test]
fn a_sessions_trail_ends_twelve_hours_after_its_newest_record() {
    use support::{account, now, put, step_tags};
    let repo = Repo::new();
    let trail = |text: &str, at: u64| {
        let mut tags = step_tags((&repo.project_id(), "repo"));
        tags.push(vec!["session".to_string(), "s1".to_string()]);
        let note = celis_wire::Note::write(&account(), 1111u16, at, tags, text).unwrap();
        put(&repo, &note);
        note
    };
    let hour = trail("an hour ago", now() - 60 * 60);
    let day = trail("a day ago", now() - 13 * 60 * 60);
    assert!(head(&repo, &hour.id).ends_with(" · active"));
    assert!(head(&repo, &day.id).ends_with(" · reached alone"));
}
