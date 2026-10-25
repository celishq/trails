//! The review `done` prints.

mod support;

use support::{account, add, cli, err, now, step_note, Repo};

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
/// next step under its `under`.
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

/// `done --changed` prints what changed, and asks to reshape the plan before the next step.
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

/// A trail's review names no next move.
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

/// `done` on an ask prints the end to fill and give the person: a line for each decision, each step whose
/// proof is the agent's own claim, what the judge could not check outside its places, each changed step,
/// the new intent and each choice, then a total of its steps, proofs by kind and minutes.
#[test]
fn done_on_an_ask_prints_the_end_to_fill() {
    let repo = Repo::new();
    let by = &account();
    let began = now() - 10 * 60 - 30;
    let trail = step_note(&repo, by, None, "the trail", began);
    let ask = step_note(&repo, by, Some(&trail), "the ask", began);
    let a = &ask.id[..8];
    let checked = add(
        &repo,
        &[
            "--under", a, "--at", "a.rs", "--check", "true", "--", "checked",
        ],
    );
    let judged = add(&repo, &["--under", a, "--at", "b.rs", "--", "judged"]);
    let claim = add(&repo, &["--under", a, "--", "claim"]);
    let gone = add(&repo, &["--under", a, "--", "gone"]);
    let edit = cli(
        &repo,
        &["step", "edit", a, "--", "the ask\nexpect: it ends"],
    );
    assert!(edit.status.success(), "{}", err(&edit));
    done(&repo, &checked, &["--as-planned"]);
    done(&repo, &judged, &["--as-planned", "--by", "judge"]);
    done(&repo, &claim, &["--changed", "it took the cache"]);
    assert!(
        cli(&repo, &["step", "drop", &gone[..8], "--", "not wanted"])
            .status
            .success()
    );
    assert_eq!(
        done(&repo, &ask.id, &["--as-planned", "--by", "person"]),
        [
            format!("celis: the ask {a} is done. Write each rule and pattern a later plan must keep with celis decide. Then answer the person with the lines below alone: fill each <…>, and remove each line you do not need."),
            String::new(),
            "**<what was done, in one line>**".to_string(),
            String::new(),
            "- [?] **Needs you:** <each decision only the person can take>".to_string(),
            "- [!] **Not proven:** claim".to_string(),
            "- [!] **Not proven:** <what the judge could not check, and each file git diff --stat shows changed outside a.rs, b.rs>".to_string(),
            "- [+] **I chose:** claim: it took the cache · <the pattern that decided it, or none>".to_string(),
            "- [+] **I chose:** <each choice you took yourself> · <the pattern that decided it, or none>".to_string(),
            String::new(),
            "celis · 4 steps · 1 checked · 1 judged · 1 person · 1 not proven · 10m".to_string(),
        ]
    );
}

/// The end of an ask whose steps share one kind of proof totals them as all of that kind.
#[test]
fn an_end_whose_steps_share_one_proof_totals_them_as_all() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let ask = add(
        &repo,
        &["--under", &trail[..8], "--check", "true", "--", "the ask"],
    );
    let part = add(
        &repo,
        &["--under", &ask[..8], "--check", "true", "--", "part"],
    );
    done(&repo, &part, &["--as-planned"]);
    let lines = done(&repo, &ask, &["--as-planned"]);
    assert_eq!(
        lines.last().map(String::as_str),
        Some("celis · 2 steps · all checked · 0m")
    );
}

/// `done --by` takes only `judge` or `person` (`spec/03-steps.md`, "Closing").
#[test]
fn done_by_takes_judge_or_person() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let o = cli(
        &repo,
        &[
            "step",
            "done",
            &trail[..8],
            "--as-planned",
            "--by",
            "robot",
            "--",
            "p",
        ],
    );
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
}
