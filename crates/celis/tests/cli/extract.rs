//! What `celis extract` prints.

use crate::support;

use support::{account, add, cli, err, mark_note, now, out, Repo};

/// The id `celis pattern -- <text>` prints.
fn pattern(repo: &Repo, text: &str) -> String {
    let o = cli(repo, &["pattern", "--", text]);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().next().unwrap().to_string()
}

/// `extract` of an open ask prints its learn brief: each prompt on its trail since the ask began, then
/// each pattern a step of the ask's tree uses, the ask too, once each (`spec/02-cli.md`, "Extract";
/// `spec/02-cli.md`, "Step").
#[test]
fn extract_of_an_open_ask_prints_its_learn_brief() {
    let repo = Repo::new();
    let (a, flat, cents) = an_ask(&repo);
    let o = cli(&repo, &["extract", &a]);
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

/// `extract` of an id that no step has refuses with `no-step` (`spec/02-cli.md`, "Output and errors").
#[test]
fn extract_of_no_step_refuses() {
    let repo = Repo::new();
    support::refusal(
        &cli(&repo, &["extract", "abcdefgh", "--json"]),
        1,
        "no-step",
    );
}

/// An open ask with a prompt, and a step under it that uses two patterns: the ask's id8 and the two
/// pattern ids.
fn an_ask(repo: &Repo) -> (String, String, String) {
    let flat = pattern(repo, "When a logo is drawn: keep it flat.");
    let cents = pattern(repo, "When prices change: keep them in cents.");
    let ask = add(repo, &["--uses", &flat[..8], "--", "Draw the logo"]);
    let a = &ask[..8];
    mark_note(
        repo,
        &account(),
        "prompt",
        &ask,
        "draw a red logo",
        now() - 5,
    );
    add(
        repo,
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
    (a.to_string(), flat, cents)
}

/// `extract --json` prints the brief as one object (`spec/02-cli.md`, "Extract").
#[test]
fn extract_json_prints_the_brief_as_one_object() {
    let repo = Repo::new();
    let (a, flat, cents) = an_ask(&repo);
    let o = cli(&repo, &["extract", &a, "--json"]);
    assert!(o.status.success(), "{}", err(&o));
    let brief: serde_json::Value = serde_json::from_str(out(&o).trim()).expect("one object");
    assert_eq!(&brief["ask"].as_str().unwrap()[..8], a);
    assert_eq!(brief["changed"], serde_json::json!([]));
    assert_eq!(brief["notes"], serde_json::json!([]));
    assert_eq!(brief["asked"], serde_json::json!(["draw a red logo"]));
    assert_eq!(
        brief["patterns"],
        serde_json::json!([
            {"id": flat, "when": "When a logo is drawn"},
            {"id": cents, "when": "When prices change"},
        ])
    );
}

/// `lesson`, which older plugins give, runs as `extract` and names itself deprecated
/// (`spec/02-cli.md`, "Output and errors").
#[test]
fn lesson_runs_as_extract_with_a_notice() {
    let repo = Repo::new();
    let (a, _, _) = an_ask(&repo);
    let old = cli(&repo, &["lesson", &a]);
    assert!(old.status.success(), "{}", err(&old));
    assert_eq!(out(&old), out(&cli(&repo, &["extract", &a])));
    assert_eq!(
        err(&old),
        "celis: lesson is deprecated; run celis extract\n"
    );
}

/// The brief gives each `expect:`, `for:` and `not:` line of the ask's text first, and `--json` gives
/// them as `expect` (`spec/02-cli.md`, "Extract"; `spec/02-cli.md`, "Step").
#[test]
fn extract_gives_what_the_ask_expected() {
    let repo = Repo::new();
    let text =
        "Draw the logo\nexpect: a red logo\nfor: the shop sign\nnot: no gradient\nask me: none";
    let ask = add(&repo, &["--", text]);
    let shown = out(&cli(&repo, &["extract", &ask[..8]]));
    let lines: Vec<&str> = shown.lines().skip(1).collect();
    assert_eq!(
        lines,
        [
            "  expect  expect: a red logo",
            "  expect  for: the shop sign",
            "  expect  not: no gradient",
        ],
        "{shown}"
    );
    let o = cli(&repo, &["extract", &ask[..8], "--json"]);
    let brief: serde_json::Value = serde_json::from_str(out(&o).trim()).expect("one object");
    assert_eq!(
        brief["expect"],
        serde_json::json!([
            "expect: a red logo",
            "for: the shop sign",
            "not: no gradient"
        ])
    );
}
