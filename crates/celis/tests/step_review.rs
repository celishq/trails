//! CIP-12, "Reviewing".

mod support;

use celis_wire::Note;
use support::{account, add, cli, err, now, put, step_note, Repo};

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

/// The stderr lines of `celis step done <id> <how> -- proof`, which must succeed.
fn done(repo: &Repo, id: &str, how: &[&str]) -> Vec<String> {
    let mut args = vec!["step", "done", &id[..8]];
    args.extend(how);
    args.extend(["--", "proof"]);
    let o = cli(repo, &args);
    assert!(o.status.success(), "{}", err(&o));
    err(&o).lines().map(str::to_string).collect()
}

/// `done` as planned prints the step's expect line and the open steps beside it, then asks for the
/// next step under its `under` (`spec/trails/12-steps.md`, "Reviewing").
#[test]
fn done_as_planned_reviews_the_step_and_names_the_next_move() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let ask = add(&repo, &["--under", &trail[..8], "--", "the ask"]);
    let piece = add(
        &repo,
        &["--under", &ask[..8], "--", "piece\nexpect: it prints 3"],
    );
    let next = add(&repo, &["--under", &ask[..8], "--", "next"]);
    let (a, p, n) = (&ask[..8], &piece[..8], &next[..8]);
    assert_eq!(
        done(&repo, &piece, &["--as-planned"]),
        [
            format!("celis: {p} is done; compare the result with its plan:"),
            "  expect  it prints 3".to_string(),
            format!("  open    {n} next"),
            format!("celis: take an open step beside it, add one step under {a}, or close {a} when its work is done."),
        ]
    );
}

/// `done --changed` prints what changed, and asks to reshape the plan before the next step
/// (`spec/trails/12-steps.md`, "Reviewing").
#[test]
fn done_changed_asks_to_reshape_the_plan() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let ask = add(&repo, &["--under", &trail[..8], "--", "the ask"]);
    let piece = add(&repo, &["--under", &ask[..8], "--", "piece"]);
    let (a, p) = (&ask[..8], &piece[..8]);
    assert_eq!(
        done(&repo, &piece, &["--changed", "it printed 4"]),
        [
            format!("celis: {p} is done; compare the result with its plan:"),
            format!("  changed {p} it printed 4"),
            format!("celis: it went otherwise than planned; replace or drop each open step it voids, then add one step under {a}."),
        ]
    );
}

/// A trail's review names no next move (`spec/trails/12-steps.md`, "Reviewing").
#[test]
fn a_trails_review_names_no_next_move() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    assert_eq!(
        done(&repo, &trail, &["--as-planned"]),
        [format!(
            "celis: {} is done; compare the result with its plan:",
            &trail[..8]
        )]
    );
}

/// `done` on an ask reviews the steps under it: those planned before the work began, those added,
/// dropped and changed, and the person's prompts since the ask; then it asks for rules
/// (`spec/trails/12-steps.md`, "Reviewing").
#[test]
fn done_on_an_ask_reviews_its_plan_and_asks_for_rules() {
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
    done(&repo, &third.id, &["--as-planned"]);
    let short = |n: &Note| n.id[..8].to_string();
    assert_eq!(
        done(&repo, &ask.id, &["--changed", "one part fewer"]),
        [
            format!(
                "celis: {} is done; compare the result with its plan:",
                short(&ask)
            ),
            "  planned 2 steps".to_string(),
            format!("  added   {} third part", short(&third)),
            format!("  dropped {} second part · not wanted", short(&second)),
            format!("  changed {} it reads the cache", short(&first)),
            format!("  changed {} one part fewer", short(&ask)),
            "  asked   use the cache".to_string(),
            "celis: write what a later plan should keep as rules with celis decide.".to_string(),
        ]
    );
}
