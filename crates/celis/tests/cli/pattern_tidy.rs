//! CIP-02, "Patterns": `celis patterns --tidy`, and the `misled` count of CIP-06, "Revisions".

use crate::support;

use support::{account, add, cli, err, finish, out, refusal, stored_pattern, Repo, DAY};

/// The id `pattern -- <text>` prints.
fn decide(repo: &Repo, args: &[&str]) -> String {
    let mut all = vec!["pattern"];
    all.extend(args);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().next().unwrap().to_string()
}

/// What `patterns --tidy` prints.
fn tidy(repo: &Repo) -> String {
    let o = cli(repo, &["patterns", "--tidy"]);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).trim_end().to_string()
}

/// A done ask with a done step under it that uses `pattern`, and a later step that fixes that step; the
/// fixed step's id.
fn misled(repo: &Repo, pattern: &str, n: usize) -> String {
    let ask = add(repo, &["--", &format!("Print price {n}")]);
    let text = format!("print price {n} in cents");
    let step = add(
        repo,
        &["--under", &ask[..8], "--uses", &pattern[..8], "--", &text],
    );
    finish(repo, &step[..8]);
    finish(repo, &ask[..8]);
    let fix = format!("Print price {n} with its cents");
    add(repo, &["--replaces", &step[..8], "--", &fix]);
    step
}

/// A pattern whose two using steps a later step fixed reads as misled twice: `patterns --tidy` lists it
/// as `misled`, the `grep` view shows ` misled:2`, and `stats` shows it (`spec/06-reuse.md`,
/// "Revisions"; `spec/02-cli.md`, "Patterns"; `spec/02-cli.md`, "Grep").
#[test]
fn a_pattern_whose_uses_were_fixed_twice_needs_tidy_as_misled() {
    let repo = Repo::new();
    let id = decide(&repo, &["--", "When a price prints: use cents."]);
    misled(&repo, &id, 1);
    let nameless = format!(
        "{} nameless When a price prints\n1 pattern in the index",
        &id[..8]
    );
    assert_eq!(tidy(&repo), nameless);
    misled(&repo, &id, 2);
    let want = format!(
        "{} misled,nameless When a price prints\n1 pattern in the index",
        &id[..8]
    );
    assert_eq!(tidy(&repo), want);
    let grep = out(&cli(&repo, &["grep", " pattern "]));
    assert!(grep.contains(" uses:2 misled:2 "), "{grep}");
    let stats = out(&cli(&repo, &["stats"]));
    assert!(stats.contains(" misled:2 "), "{stats}");
}

/// An ask that used a pattern and was then fixed misled it once, and the `--recalled` that older plugins
/// give counts for nothing (`spec/06-reuse.md`, "Revisions").
#[test]
fn a_fixed_ask_that_used_a_pattern_misled_it_once() {
    let repo = Repo::new();
    let id = decide(&repo, &["--", "When a price prints: use cents."]);
    let ask = add(
        &repo,
        &[
            "--recalled",
            &id[..8],
            "--uses",
            &id[..8],
            "--",
            "Print the prices",
        ],
    );
    finish(&repo, &ask[..8]);
    add(
        &repo,
        &["--replaces", &ask[..8], "--", "Print the prices in cents"],
    );
    let grep = out(&cli(&repo, &["grep", " pattern "]));
    assert!(grep.contains(" uses:1 misled:1 "), "{grep}");
}

/// Each of two live patterns whose Whens share 2 or more words lists the other as `near`
/// (`spec/02-cli.md`, "Patterns").
#[test]
fn near_copies_name_each_other() {
    let repo = Repo::new();
    let a = decide(
        &repo,
        &["--", "When code handles money amounts: keep cents."],
    );
    let b = decide(&repo, &["--", "When you store money amounts: use cents."]);
    decide(&repo, &["--", "When you indent Python: use spaces."]);
    let near = |x: &str, y: &str| format!("{} near:{},nameless ", &x[..8], &y[..8]);
    let listed = tidy(&repo);
    let lines: Vec<&str> = listed.lines().filter(|l| !l.contains("Python")).collect();
    assert_eq!(lines.len(), 3, "{listed}");
    assert!(
        lines.iter().any(|l| l.starts_with(&near(&a, &b))),
        "{listed}"
    );
    assert!(
        lines.iter().any(|l| l.starts_with(&near(&b, &a))),
        "{listed}"
    );
    assert_eq!(lines[2], "3 patterns in the index");
}

/// A pattern made 60 or more days ago that no step used in the last 60 days is `unused`, only while a
/// step of the project used a pattern (`spec/02-cli.md`, "Patterns").
#[test]
fn an_old_pattern_no_step_uses_is_unused() {
    let repo = Repo::new();
    let project = (repo.project_id(), "repo".to_string());
    let old = stored_pattern(
        &repo,
        (&project.0, &project.1),
        &account(),
        (None, None),
        "When a release is cut: tag it.",
        support::now() - 61 * DAY,
    );
    let fresh = decide(&repo, &["--", "When a logo is drawn: keep it flat."]);
    let line = |reasons: &str| format!("{} {reasons} When a release is cut", &old.id[..8]);
    let logo = format!("{} nameless When a logo is drawn", &fresh[..8]);
    let none = format!("{}\n{logo}\n2 patterns in the index", line("nameless"));
    assert_eq!(tidy(&repo), none);
    add(&repo, &["--uses", &fresh[..8], "--", "Draw the logo"]);
    let want = format!(
        "{}\n{logo}\n2 patterns in the index",
        line("unused,nameless")
    );
    assert_eq!(tidy(&repo), want);
}

/// A pattern at the end of a chain of 3 `replaces` is `churn`, and one of 2 is not
/// (`spec/02-cli.md`, "Patterns").
#[test]
fn a_pattern_replaced_three_times_is_churn() {
    let repo = Repo::new();
    let mut id = decide(&repo, &["--", "When a logo is drawn: keep it flat."]);
    for n in 1..=3 {
        let want = format!(
            "{} nameless When a logo is drawn\n1 pattern in the index",
            &id[..8]
        );
        assert_eq!(tidy(&repo), want);
        let text = format!("When a logo is drawn: keep it flat, take {n}.");
        id = decide(&repo, &["--replaces", &id[..8], "--", &text]);
    }
    let want = format!(
        "{} churn,nameless When a logo is drawn\n1 pattern in the index",
        &id[..8]
    );
    assert_eq!(tidy(&repo), want);
}

/// `patterns --tidy` refuses a `<file>` and `--markdown` as usage errors (`spec/02-cli.md`, "Patterns").
#[test]
fn tidy_takes_no_file_and_no_markdown() {
    let repo = Repo::new();
    refusal(
        &cli(&repo, &["patterns", "--json", "--tidy", "a.py"]),
        2,
        "usage",
    );
    refusal(
        &cli(&repo, &["patterns", "--json", "--tidy", "--markdown"]),
        2,
        "usage",
    );
    assert_eq!(tidy(&repo), "0 patterns in the index");
}

/// A pattern misled a step once when the step's `done` names it in `misled`, and when the step that
/// fixes it names it in `misled`; a step counts once (`spec/06-reuse.md`, "Misled").
#[test]
fn a_misled_mark_counts_once_for_each_step() {
    let repo = Repo::new();
    let id = decide(&repo, &["--", "When a price prints: use cents."]);
    let ask = add(&repo, &["--", "Print the price"]);
    let o = cli(
        &repo,
        &[
            "step",
            "done",
            &ask[..8],
            "--as-planned",
            "--misled",
            &id[..8],
            "--",
            "it prints",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let fix = [
        "--replaces",
        &ask[..8],
        "--misled",
        &id[..8],
        "--",
        "Print it right",
    ];
    add(&repo, &fix);
    let stats = out(&cli(&repo, &["stats"]));
    assert!(stats.contains(" misled:1 "), "{stats}");
    let other = add(&repo, &["--", "Print the tax"]);
    finish(&repo, &other[..8]);
    let fix = [
        "--replaces",
        &other[..8],
        "--misled",
        &id[..8],
        "--",
        "Print the tax right",
    ];
    add(&repo, &fix);
    let stats = out(&cli(&repo, &["stats"]));
    assert!(stats.contains(" misled:2 "), "{stats}");
}

/// `step add --misled` refuses an id that names no pattern with `no-pattern`, and a step that fixes no
/// done step as a usage error (`spec/02-cli.md`, "Step").
#[test]
fn misled_needs_a_pattern_and_a_fix() {
    let repo = Repo::new();
    let id = decide(&repo, &["--", "When a price prints: use cents."]);
    let ask = add(&repo, &["--", "Print the price"]);
    let o = cli(
        &repo,
        &["step", "add", "--json", "--misled", &id[..8], "--", "Print"],
    );
    refusal(&o, 2, "usage");
    finish(&repo, &ask[..8]);
    let fix = ["--replaces", &ask[..8], "--misled", "abcd1234", "--", "Fix"];
    let mut args = vec!["step", "add", "--json"];
    args.extend(fix);
    refusal(&cli(&repo, &args), 1, "no-pattern");
}

/// A pattern with no Name is `nameless`, and one whose When starts with `When:` or holds another of its
/// lines is `form`; a named pattern in the right form has neither (`spec/02-cli.md`, "Patterns").
#[test]
fn a_nameless_or_malformed_pattern_needs_tidy() {
    let repo = Repo::new();
    let named = "Name: Light on two sides\nWhen a living room is lit\nForces: one window glares\nDo: add a second window\nBecause: two sides soften shadows";
    decide(&repo, &["--", named]);
    let plain = decide(&repo, &["--", "When a price prints: use cents."]);
    let label = decide(
        &repo,
        &["--", "Name: Cents\nWhen: an invoice totals\nDo: add cents"],
    );
    let packed = decide(
        &repo,
        &[
            "--",
            "Name: Tags\nWhen a tag ships. Forces: speed. Do: sign it.\nBecause: trust",
        ],
    );
    let want = [
        format!("{} nameless When a price prints", &plain[..8]),
        format!("{} form Cents", &label[..8]),
        format!("{} form Tags", &packed[..8]),
        "4 patterns in the index".to_string(),
    ];
    let listed = tidy(&repo);
    let mut lines: Vec<&str> = listed.lines().collect();
    lines.sort();
    let mut want: Vec<&str> = want.iter().map(String::as_str).collect();
    want.sort();
    assert_eq!(lines, want, "{listed}");
}
