//! CIP-04, "Patterns".

mod support;

use support::{account, cli, ids, now, out, step_note, stored_pattern, Repo};

/// A reader counts a pattern as one of a trail's patterns only when its scope names that trail, and a
/// replaced one not at all (`spec/04-trails.md`, "Patterns").
#[test]
fn a_pattern_holds_for_a_trail_only_when_its_scope_names_it() {
    let repo = Repo::new();
    let id = repo.project_id();
    let project = (id.as_str(), "repo");
    let t = now() - 5 * 60;
    let mine = step_note(&repo, &account(), None, "mine", t);
    let other = step_note(&repo, &account(), None, "other", t + 1);
    let scope = |trail: &str| format!("trail:{trail}");
    let pattern = stored_pattern(
        &repo,
        project,
        &account(),
        (Some(&scope(&mine.id)), None),
        "rule",
        t + 2,
    );
    let old = stored_pattern(
        &repo,
        project,
        &account(),
        (Some(&scope(&mine.id)), None),
        "old",
        t + 3,
    );
    stored_pattern(
        &repo,
        project,
        &account(),
        (None, Some(&old.id)),
        "plain",
        t + 4,
    );
    stored_pattern(
        &repo,
        project,
        &account(),
        (Some(&scope(&other.id)), None),
        "theirs",
        t + 5,
    );
    let text = out(&cli(
        &repo,
        &["grep", &format!(" live {{trail:{}}}", &mine.id[..8])],
    ));
    assert_eq!(ids(&text, "pattern"), [&pattern.id[..8]]);
}
