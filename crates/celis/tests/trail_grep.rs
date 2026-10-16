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
    assert_eq!(
        grep(&repo, &["-c", "-i", "vendors"]).lines().next(),
        Some("1")
    );
}

/// Each line holds the day, the trail, the project, the kind, the id, the state and places, and the text
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
    let head = line(&text, &format!(" trail {t} "));
    assert!(
        head.ends_with(&format!("{t} [repo] trail {t} open Build the Vendors page")),
        "{head}"
    );
    assert_eq!(head.split(' ').next().map(str::len), Some(10), "{head}");
    let step = line(&text, &format!(" step {s} "));
    assert!(
        step.ends_with(&format!(
            "{t} [repo] step {s} open at:src/vendors.rs Add the table expect: it shows"
        )),
        "{step}"
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
    let t = &trail[..8];
    assert!(line(&text, "] proof ").ends_with(&format!(
        "{t} [repo] proof {} the page shows it",
        &done[..8]
    )));
    assert!(line(&text, "] changed ").ends_with(&format!(
        "[repo] changed {} a list, not a table",
        &done[..8]
    )));
    assert!(line(&text, "] dropped ")
        .ends_with(&format!("[repo] dropped {} no data for it", &gone[..8])));
    assert!(
        line(&text, &format!(" step {} ", &gone[..8])).contains(" dropped "),
        "{text}"
    );
}

/// A decision prints as a `rule`: live or replaced, then its scope; one of a trail prints the trail, one
/// of the project `-` (`spec/02-cli.md`, "Grep").
#[test]
fn grep_prints_decisions_as_rules_live_or_replaced() {
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
    let held = line(&text, "initials");
    assert!(
        held.ends_with(&format!(
            "{t} [repo] rule {} live {{trail:{t}}} When a vendor has no logo: show its initials",
            &rule[..8]
        )),
        "{held}"
    );
    let gone = line(&text, "floats");
    assert!(
        gone.ends_with(&format!(
            "- [repo] rule {} replaced-by {} {{project}} When a price prints: use floats",
            &old[..8],
            &new[..8]
        )),
        "{gone}"
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
    assert!(first.ends_with("Fix the login"), "{text}");
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

/// After the lines, `grep` names the working folder's project and each trail of them once
/// (`spec/02-cli.md`, "Grep").
#[test]
fn grep_ends_with_the_project_and_a_legend_of_trails() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "Build the Vendors page"]);
    add(
        &repo,
        &["--under", &trail[..8], "--", "Add the vendor table"],
    );
    let text = grep(&repo, &["vendor"]);
    let legend: Vec<&str> = text
        .lines()
        .skip_while(|l| *l != "project [repo]")
        .collect();
    assert_eq!(legend.get(1), Some(&"trails"), "{text}");
    let day = text.split(' ').next().unwrap();
    assert_eq!(
        legend[2..],
        [format!(
            "{} [repo] Build the Vendors page · {day}",
            &trail[..8]
        )],
        "{text}"
    );
}

/// Past 14,000 bytes of lines, `grep` cuts them, counts what it cut and lists each trail it cut, and
/// stays within 20,000 bytes (`spec/02-cli.md`, "Grep").
#[test]
fn grep_cuts_long_output_and_lists_the_trails_it_cut() {
    let repo = Repo::new();
    let long = "a long title that says the same thing again and again ".repeat(4);
    for n in 0..40 {
        let trail = add(&repo, &["--new", "--", &format!("Trail {n} {long}")]);
        for m in 0..3 {
            add(
                &repo,
                &["--under", &trail[..8], "--", &format!("Step {m} {long}")],
            );
        }
    }
    let text = grep(&repo, &["long title"]);
    assert!(text.len() <= 20_000, "{}", text.len());
    let cut = line(&text, "celis: ");
    assert!(
        cut.contains(" more lines in ") && cut.contains(" trails match;"),
        "{cut}"
    );
    assert!(text.contains(" · 3 steps"), "{text}");
    assert!(text.lines().any(|l| l == "trails"), "{text}");
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

/// `celis grep --help` prints the command page (`spec/02-cli.md`, "Grep").
#[test]
fn grep_help_prints_the_command_page() {
    let repo = Repo::new();
    let text = grep(&repo, &["--help"]);
    assert!(text.starts_with("celis grep"), "{text}");
}

/// The legend names the trails of lines that `grep` numbers with `-n` (`spec/02-cli.md`, "Grep").
#[test]
fn grep_names_the_trails_of_numbered_lines() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "Build the Vendors page"]);
    let text = grep(&repo, &["-n", "Vendors"]);
    assert!(text.starts_with("1:"), "{text}");
    let legend: Vec<&str> = text.lines().skip_while(|l| *l != "trails").collect();
    assert!(
        legend[1].starts_with(&format!("{} [repo] Build the Vendors page", &trail[..8])),
        "{text}"
    );
}

/// `past` writes nothing and prints the `grep` searches that replace it, with the project's name, so an
/// old plugin's agent goes on (`spec/02-cli.md`, "Past").
#[test]
fn past_prints_the_searches_that_replace_it() {
    let repo = Repo::new();
    let o = cli(&repo, &["past", "--at", "src/a.rs"]);
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&o);
    assert!(
        text.starts_with("celis past is gone; search the trails with celis grep:"),
        "{text}"
    );
    assert!(text.contains("celis grep \" live {me} \""), "{text}");
    assert!(
        text.contains("celis grep \"\\[repo\\] rule [0-9a-f]* live {project} \""),
        "{text}"
    );
    assert!(!repo.store().exists());
}

/// With `-q`, a match prints nothing, and no match still says so (`spec/02-cli.md`, "Grep").
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
