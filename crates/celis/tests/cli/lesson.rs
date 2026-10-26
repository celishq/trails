//! What `celis lesson` prints.

use crate::support;

use support::{account, add, cli, err, mark_note, now, out, Repo};

/// The id `celis pattern -- <text>` prints.
fn pattern(repo: &Repo, text: &str) -> String {
    let o = cli(repo, &["pattern", "--", text]);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().next().unwrap().to_string()
}

/// `lesson` of an open ask prints its learn brief: each prompt on its trail since the ask began, then
/// each pattern a step of the ask's tree uses, the ask too, once each (`spec/02-cli.md`, "Lesson";
/// `spec/02-cli.md`, "Step").
#[test]
fn lesson_of_an_open_ask_prints_its_learn_brief() {
    let repo = Repo::new();
    let flat = pattern(&repo, "When a logo is drawn: keep it flat.");
    let cents = pattern(&repo, "When prices change: keep them in cents.");
    let ask = add(&repo, &["--uses", &flat[..8], "--", "Draw the logo"]);
    let a = &ask[..8];
    mark_note(
        &repo,
        &account(),
        "prompt",
        &ask,
        "draw a red logo",
        now() - 5,
    );
    add(
        &repo,
        &[
            "--under",
            a,
            "--uses",
            &cents[..8],
            "--uses",
            &flat[..8],
            "--",
            "the price tag",
        ],
    );
    let o = cli(&repo, &["lesson", a]);
    assert!(o.status.success(), "{}", err(&o));
    let shown = out(&o);
    let lines: Vec<&str> = shown.lines().collect();
    assert!(
        lines[0].starts_with(&format!("celis: before you answer, learn from {a}.")),
        "{shown}"
    );
    assert_eq!(
        lines[1..],
        [
            "  asked   draw a red logo".to_string(),
            format!("  pattern {} When a logo is drawn", &flat[..8]),
            format!("  pattern {} When prices change", &cents[..8]),
        ]
    );
}

/// `lesson` of an id that no step has refuses with `no-step` (`spec/02-cli.md`, "Output and errors").
#[test]
fn lesson_of_no_step_refuses() {
    let repo = Repo::new();
    support::refusal(&cli(&repo, &["lesson", "abcdefgh", "--json"]), 1, "no-step");
}
