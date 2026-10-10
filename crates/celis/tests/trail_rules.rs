//! CIP-14, "Rules".

mod support;

use support::{account, decision, now, objects, step_note, Repo, DAY};

/// A reader counts a decision as a rule of a trail only when its scope names that trail, and a replaced
/// one not at all (`spec/trails/14-trails.md`, "Rules").
#[test]
fn a_decision_is_a_rule_of_a_trail_only_when_its_scope_names_it() {
    let repo = Repo::new();
    let id = repo.project_id();
    let project = (id.as_str(), "repo");
    let t = now() - 5 * DAY;
    let mine = step_note(&repo, &account(), None, "mine", t);
    let other = step_note(&repo, &account(), None, "other", t + 1);
    let scope = |trail: &str| format!("trail:{trail}");
    let rule = decision(
        &repo,
        project,
        &account(),
        (Some(&scope(&mine.id)), None),
        "rule",
        t + 2,
    );
    let old = decision(
        &repo,
        project,
        &account(),
        (Some(&scope(&mine.id)), None),
        "old",
        t + 3,
    );
    decision(
        &repo,
        project,
        &account(),
        (None, Some(&old.id)),
        "plain",
        t + 4,
    );
    decision(
        &repo,
        project,
        &account(),
        (Some(&scope(&other.id)), None),
        "theirs",
        t + 5,
    );
    let rules: Vec<String> = objects(&repo, &["read"])
        .iter()
        .filter_map(|v| v.get("trail"))
        .find(|t| t["id"] == mine.id.as_str())
        .expect("the trail")["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(rules, [rule.id]);
}
