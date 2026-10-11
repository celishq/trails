//! CIP-14, "Rules".

mod support;

use celis_wire::Note;
use support::{account, cli, decision, err, now, objects, put, step_note, Repo};

/// A reader counts a decision as a rule of a trail only when its scope names that trail, and a replaced
/// one not at all (`spec/trails/14-trails.md`, "Rules").
#[test]
fn a_decision_is_a_rule_of_a_trail_only_when_its_scope_names_it() {
    let repo = Repo::new();
    let id = repo.project_id();
    let project = (id.as_str(), "repo");
    let t = now() - 5 * 60;
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

/// A mark `label` on `step`, saying `content`, signed at `at`, put in the store; with `changed`, a done
/// that says what changed.
fn mark(repo: &Repo, label: &str, step: &Note, content: &str, changed: Option<&str>, at: u64) {
    let mut tags: Vec<Vec<String>> = vec![
        vec!["L".into(), "celis.step".into()],
        vec!["l".into(), label.into(), "celis.step".into()],
        vec!["e".into(), step.id.clone()],
    ];
    tags.extend(changed.map(|c| vec!["changed".to_string(), c.to_string()]));
    put(
        repo,
        &Note::write(&account(), 1985u16, at, tags, content).unwrap(),
    );
}

/// `done` on an ask prints its retro on stderr: the steps planned before the work began, those added,
/// dropped and changed, and the person's prompts, and asks for rules; a deeper step prints none
/// (`spec/trails/14-trails.md`, "Rules").
#[test]
fn done_on_an_ask_prints_its_retro() {
    let repo = Repo::new();
    let by = &account();
    let t = now() - 10 * 60;
    let trail = step_note(&repo, by, None, "the trail", t);
    mark(&repo, "prompt", &trail, "an older ask", None, t);
    let ask = step_note(&repo, by, Some(&trail), "the ask", t + 1);
    let first = step_note(&repo, by, Some(&ask), "first part", t + 2);
    let second = step_note(&repo, by, Some(&ask), "second part", t + 3);
    mark(&repo, "start", &first, "", None, t + 10);
    mark(&repo, "prompt", &trail, "use the cache", None, t + 15);
    let third = step_note(&repo, by, Some(&ask), "third part", t + 20);
    mark(&repo, "drop", &second, "not wanted", None, t + 25);
    mark(
        &repo,
        "done",
        &first,
        "p",
        Some("it reads the cache"),
        t + 30,
    );
    let o = cli(
        &repo,
        &["step", "done", &third.id[..8], "--as-planned", "--", "p"],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(err(&o), "");
    let o = cli(
        &repo,
        &[
            "step",
            "done",
            &ask.id[..8],
            "--changed",
            "one part fewer",
            "--",
            "p",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let short = |n: &Note| n.id[..8].to_string();
    let expected = [
        format!("celis: the ask {} is done; write what a later plan should keep as rules with celis decide:", short(&ask)),
        "  planned 2 steps".to_string(),
        format!("  added   {} third part", short(&third)),
        format!("  dropped {} second part · not wanted", short(&second)),
        format!("  changed {} it reads the cache", short(&first)),
        format!("  changed {} one part fewer", short(&ask)),
        "  asked   use the cache".to_string(),
    ];
    assert_eq!(err(&o).lines().collect::<Vec<_>>(), expected);
}
