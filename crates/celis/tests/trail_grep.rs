//! CIP-02, "Grep".

mod support;

use std::time::Duration;
use support::{add, cli, err, finish, out, Repo};

/// The stdout of `celis grep <args>` in the main checkout, which must exit 0.
fn grep(repo: &Repo, args: &[&str]) -> String {
    let mut all = vec!["grep"];
    all.extend(args);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o)
}

/// The line of `text` that holds `part`.
fn line<'a>(text: &'a str, part: &str) -> &'a str {
    text.lines()
        .find(|l| l.contains(part))
        .unwrap_or_else(|| panic!("no line with {part:?} in {text}"))
}

/// `grep` gives its words to the system `grep`, so its options work (`spec/02-cli.md`, "Grep").
#[test]
fn grep_gives_its_words_to_grep() {
    let repo = Repo::new();
    add(&repo, &["--", "Build the Vendors page"]);
    add(&repo, &["--", "Fix the login"]);
    let text = grep(&repo, &["-i", "VENDORS"]);
    assert!(text.contains("Build the Vendors page"), "{text}");
    assert!(!text.contains("Fix the login"), "{text}");
    assert_eq!(grep(&repo, &["-c", "-i", "vendors"]).trim(), "1");
}

/// Each line prints under its trail's header with the kind, the id, the state and places, and the text
/// (`spec/02-cli.md`, "Grep").
#[test]
fn grep_prints_a_trail_and_its_step_as_lines() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "Build the Vendors page"]);
    let step = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--at",
            "src/vendors.rs",
            "--",
            "Add the table\nexpect: it shows",
        ],
    );
    let text = grep(&repo, &["Vendors\\|table"]);
    let (t, s) = (&trail[..8], &step[..8]);
    let header = text.lines().next().unwrap();
    assert!(
        header.starts_with(&format!("{t} [repo] Build the Vendors page · ")),
        "{text}"
    );
    assert_eq!(
        header.rsplit(' ').next().map(str::len),
        Some(10),
        "{header}"
    );
    assert_eq!(
        line(&text, &format!(" trail {t} ")),
        format!("  trail {t} open")
    );
    assert_eq!(
        line(&text, &format!(" step {s} ")),
        format!("  step {s} open at:src/vendors.rs Add the table expect: it shows")
    );
}

/// A done step prints its proof and its `changed` text, a dropped step its reason, under the trail
/// (`spec/02-cli.md`, "Grep").
#[test]
fn grep_prints_proofs_changes_and_drop_reasons() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "Build the Vendors page"]);
    let done = add(&repo, &["--under", &trail[..8], "--", "Add the table"]);
    let o = cli(
        &repo,
        &[
            "step",
            "done",
            &done[..8],
            "--changed",
            "a list, not a table",
            "--",
            "the page shows it",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let gone = add(&repo, &["--under", &trail[..8], "--", "Add the chart"]);
    let o = cli(&repo, &["step", "drop", &gone[..8], "--", "no data for it"]);
    assert!(o.status.success(), "{}", err(&o));
    let text = grep(&repo, &[""]);
    assert_eq!(
        line(&text, "  proof "),
        format!("  proof {} the page shows it", &done[..8])
    );
    assert_eq!(
        line(&text, "  changed "),
        format!("  changed {} a list, not a table", &done[..8])
    );
    assert_eq!(
        line(&text, "  dropped "),
        format!("  dropped {} no data for it", &gone[..8])
    );
    assert!(
        line(&text, &format!(" step {} ", &gone[..8])).contains(" dropped "),
        "{text}"
    );
}

/// A rule prints with the kind `rule`: live or replaced, then its scope; one of a trail prints under
/// the trail, one of the project under `- rules of no trail` with its project (`spec/02-cli.md`, "Grep").
#[test]
fn grep_prints_rules_live_or_replaced() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "Build the Vendors page"]);
    let decide = |args: &[&str]| {
        let mut all = vec!["decide"];
        all.extend(args);
        let o = cli(&repo, &all);
        assert!(o.status.success(), "{}", err(&o));
        out(&o).trim().to_string()
    };
    let scope = format!("trail:{}", &trail[..8]);
    let rule = decide(&[
        "--scope",
        &scope,
        "--",
        "When a vendor has no logo: show its initials",
    ]);
    let old = decide(&["--", "When a price prints: use floats"]);
    let new = decide(&[
        "--replaces",
        &old[..8],
        "--",
        "When a price prints: use cents",
    ]);
    let text = grep(&repo, &[" rule "]);
    let t = &trail[..8];
    assert_eq!(
        line(&text, "initials"),
        format!(
            "  rule {} live {{trail:{t}}} When a vendor has no logo: show its initials",
            &rule[..8]
        )
    );
    assert!(text.lines().any(|l| l == "- rules of no trail"), "{text}");
    assert_eq!(
        line(&text, "floats"),
        format!(
            "  [repo] rule {} replaced-by {} {{project}} When a price prints: use floats",
            &old[..8],
            &new[..8]
        )
    );
    assert!(
        line(&text, "cents").contains(&format!(" rule {} live {{project}} ", &new[..8])),
        "{text}"
    );
}

/// The view prints the newest trail first (`spec/02-cli.md`, "Grep").
#[test]
fn grep_prints_the_newest_trail_first() {
    let repo = Repo::new();
    add(&repo, &["--", "Build the Vendors page"]);
    std::thread::sleep(Duration::from_millis(1100));
    add(&repo, &["--new", "--", "Fix the login"]);
    let text = grep(&repo, &[" trail "]);
    let first = text.lines().next().unwrap();
    assert!(first.contains("] Fix the login · "), "{text}");
}

/// The trail with the most lines found prints first (`spec/02-cli.md`, "Grep").
#[test]
fn grep_prints_the_trail_with_most_lines_first() {
    let repo = Repo::new();
    let old = add(&repo, &["--", "Build the vendor page"]);
    add(&repo, &["--under", &old[..8], "--", "Add the vendor table"]);
    add(&repo, &["--under", &old[..8], "--", "Add the vendor chart"]);
    std::thread::sleep(Duration::from_millis(1100));
    add(&repo, &["--new", "--", "Fix the vendor login"]);
    let text = grep(&repo, &["vendor"]);
    let first = text.lines().next().unwrap();
    assert!(first.contains("] Build the vendor page · "), "{text}");
}

/// A done step that a later step replaces prints `revised-by` (`spec/02-cli.md`, "Grep").
#[test]
fn grep_prints_who_revised_a_done_step() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "Build the Vendors page"]);
    let first = add(&repo, &["--under", &trail[..8], "--", "Add the table"]);
    finish(&repo, &first[..8]);
    let again = add(
        &repo,
        &["--replaces", &first[..8], "--", "Add the table again"],
    );
    let text = grep(&repo, &[&format!("step {}", &first[..8])]);
    assert!(
        text.contains(&format!(" done revised-by {} Add the table", &again[..8])),
        "{text}"
    );
}

/// The lines print under their trail's header, then the working folder's project, with no legend
/// (`spec/02-cli.md`, "Grep").
#[test]
fn grep_ends_with_the_project_and_no_legend() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "Build the Vendors page"]);
    add(
        &repo,
        &["--under", &trail[..8], "--", "Add the vendor table"],
    );
    let text = grep(&repo, &["-i", "vendor"]);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 5, "{text}");
    assert!(
        lines[1].starts_with("  trail ") && lines[2].starts_with("  step "),
        "{text}"
    );
    assert_eq!(lines[3..], ["", "project [repo]"], "{text}");
}

/// A line of `grep` that is no line of the view, such as `--`, does not print (`spec/02-cli.md`,
/// "Grep").
#[test]
fn grep_drops_its_group_separators() {
    let repo = Repo::new();
    add(&repo, &["--", "Build the Vendors page"]);
    add(&repo, &["--new", "--", "Fix the login"]);
    add(&repo, &["--new", "--", "Build the Vendors list"]);
    let text = grep(&repo, &["-A0", "Vendors"]);
    assert!(!text.lines().any(|l| l == "--"), "{text}");
    assert_eq!(
        text.lines().filter(|l| l.starts_with("  trail ")).count(),
        2
    );
}

/// `grep` prints every line it found, however long the output, and adds no line about a cut
/// (`spec/02-cli.md`, "Grep").
#[test]
fn grep_prints_every_line_however_long() {
    let repo = Repo::new();
    let long = "the same long rule text again and again ".repeat(5);
    let long = long.trim();
    for n in 0..60 {
        let o = cli(&repo, &["decide", "--", &format!("When {n}: {long}")]);
        assert!(o.status.success(), "{}", err(&o));
    }
    let text = grep(&repo, &[" rule "]);
    assert!(text.len() > 8_000, "{}", text.len());
    assert_eq!(
        text.lines().filter(|l| l.contains(long)).count(),
        60,
        "{text}"
    );
    assert!(!text.contains("celis: "), "{text}");
    assert!(text.trim_end().ends_with("project [repo]"), "{text}");
}

/// The last line `grep` finds prints even when its text ends in a space (`spec/02-cli.md`, "Grep").
#[test]
fn grep_prints_a_last_line_that_ends_in_a_space() {
    let repo = Repo::new();
    let o = cli(&repo, &["decide", "--", "When a page loads: keep it fast "]);
    assert!(o.status.success(), "{}", err(&o));
    let text = grep(&repo, &["keep it fast"]);
    assert!(text.starts_with("- rules of no trail\n"), "{text}");
    assert!(text.contains("When a page loads: keep it fast"), "{text}");
}

/// With no line that matches, `grep` says so and exits 0; when `grep` fails, it exits 2
/// (`spec/02-cli.md`, "Grep").
#[test]
fn grep_says_when_no_line_matches_and_passes_on_a_failure() {
    let repo = Repo::new();
    add(&repo, &["--", "Build the Vendors page"]);
    assert_eq!(grep(&repo, &["nothing like it"]).trim(), "No line matches.");
    let o = cli(&repo, &["grep", "--no-such-grep-option", "x"]);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    assert!(!err(&o).contains("celis:"), "{}", err(&o));
}

/// `celis grep --help` prints the command page.
#[test]
fn grep_help_prints_the_command_page() {
    let repo = Repo::new();
    let text = grep(&repo, &["--help"]);
    assert!(text.starts_with("celis grep"), "{text}");
}

/// A line that `grep` numbers with `-n` prints its number, under its trail's header
/// (`spec/02-cli.md`, "Grep").
#[test]
fn grep_prints_numbered_lines_under_their_trail() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "Build the Vendors page"]);
    let text = grep(&repo, &["-n", "Vendors"]);
    let t = &trail[..8];
    assert!(
        text.starts_with(&format!("{t} [repo] Build the Vendors page · ")),
        "{text}"
    );
    assert_eq!(
        text.lines().nth(1),
        Some(format!("  1:trail {t} open").as_str())
    );
}

/// With `-q`, a match prints nothing, and no match still says so.
#[test]
fn grep_q_prints_nothing_on_a_match() {
    let repo = Repo::new();
    add(&repo, &["--", "Build the Vendors page"]);
    assert_eq!(grep(&repo, &["-q", "Vendors"]), "");
    assert_eq!(
        grep(&repo, &["-q", "nothing like it"]).trim(),
        "No line matches."
    );
}

/// The view and what `grep` prints of it match `spec/fixtures/grep-view.json`, and the rule search of
/// the card prints exactly the lines it keeps (`spec/02-cli.md`, "Grep").
#[test]
fn the_view_and_the_cards_rule_search_match_the_fixture() {
    use celis_trails::{Anchor, Mark, StepId, StepRecord};
    use support::{account, put, stored_rule, tagged_rule};
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../spec/fixtures/grep-view.json")).unwrap();
    let card = include_str!("../../../plugins/celis/card.md");
    let repo = Repo::new();
    let (by, id) = (account(), repo.project_id());
    let t = 1_768_478_400;
    let step = |under: Option<&str>, places: &[&str], text: &str, at: u64| {
        let record = StepRecord {
            anchor: Anchor {
                project: id.clone(),
                name: "repo".to_string(),
            },
            under: under.map(StepId::new),
            replaces: None,
            uses: Vec::new(),
            places: places.iter().map(|p| p.to_string()).collect(),
            check: None,
            session: None,
            content: text.to_string(),
        };
        let note = record.record(&by, at).unwrap();
        put(&repo, &note);
        note
    };
    let mark = |mark: Mark, step: &str, at: u64| put(&repo, &mark.record(&by, step, at).unwrap());
    let trail = step(None, &[], "Build the Vendors page", t);
    let table = step(Some(&trail.id), &["src/vendors.rs"], "Add the table", t + 1);
    let done = Mark::Done {
        proof: "the table shows".into(),
        changed: Some("a list, not a table".into()),
        by: None,
        files: vec![],
    };
    mark(done, &table.id, t + 2);
    let chart = step(Some(&trail.id), &[], "Add the chart", t + 3);
    mark(
        Mark::Drop {
            reason: "no data for it".into(),
        },
        &chart.id,
        t + 4,
    );
    let project = (id.as_str(), "repo");
    let scope = format!("trail:{}", trail.id);
    let rule = stored_rule(
        &repo,
        project,
        &by,
        (Some(&scope), None),
        "When a vendor has no logo: show its initials.",
        t + 5,
    );
    let old = stored_rule(
        &repo,
        project,
        &by,
        (None, None),
        "When a price prints: use floats.",
        t + 6,
    );
    let new = stored_rule(
        &repo,
        project,
        &by,
        (None, Some(&old.id)),
        "When a price prints: use cents.",
        t + 7,
    );
    let me = stored_rule(
        &repo,
        project,
        &by,
        (Some("me"), None),
        "When you write a commit message: use plain words.",
        t + 8,
    );
    let tag = |tag: &[&str]| vec![tag.iter().map(|t| t.to_string()).collect()];
    let situation = tagged_rule(
        &repo,
        project,
        &by,
        (None, None),
        tag(&["l", "situation", "celis.decision"]),
        "When the person asks a question: answer it first.",
        t + 9,
    );
    let push = tagged_rule(
        &repo,
        project,
        &by,
        (Some("me"), None),
        tag(&["path", ".github/workflows/*.yml"]),
        "When you push: run the tests first.",
        t + 10,
    );
    let names = [
        ("trail", &trail),
        ("table", &table),
        ("chart", &chart),
        ("rule", &rule),
        ("old", &old),
        ("new", &new),
        ("me", &me),
        ("situation", &situation),
        ("push", &push),
    ];
    let fill = |line: &serde_json::Value| {
        let mut line = line.as_str().unwrap().to_string();
        for (name, note) in &names {
            line = line.replace(&format!("<{name}>"), &note.id[..8]);
        }
        line
    };
    let lines = |text: String| -> Vec<String> {
        text.lines()
            .take_while(|l| !l.is_empty())
            .map(str::to_string)
            .collect()
    };
    for want in fixture["lines"].as_array().unwrap().iter().map(fill) {
        assert_eq!(
            grep(&repo, &["-c", "-x", "-F", &want]).trim(),
            "1",
            "{want}"
        );
    }
    let want: Vec<String> = fixture["printed"]
        .as_array()
        .unwrap()
        .iter()
        .map(fill)
        .collect();
    assert_eq!(lines(grep(&repo, &[""])), want);
    let pattern = card
        .split("`celis grep -i -E \"")
        .nth(1)
        .unwrap()
        .split("\"`")
        .next()
        .unwrap()
        .replace("<word>|<word>", "price|push|question");
    let args = ["-i", "-E", pattern.as_str()];
    let kept = lines(grep(&repo, &args));
    let want: Vec<String> = fixture["kept"]
        .as_array()
        .unwrap()
        .iter()
        .map(fill)
        .collect();
    assert_eq!(kept, want);
}
