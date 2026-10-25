//! CIP-02, "Recall".

mod support;

use support::{add, cli, err, out, Repo};

/// The id `celis decide <args> -- <text>` prints.
fn decide(repo: &Repo, args: &[&str], text: &str) -> String {
    let mut all = vec!["decide"];
    all.extend(args);
    all.extend(["--", text]);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().next().unwrap().to_string()
}

/// What `celis recall <args>` prints.
fn recall(repo: &Repo, args: &[&str]) -> String {
    let mut all = vec!["recall"];
    all.extend(args);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o)
}

/// A store of 3 rules and 1 pattern, and the ids of the rule of `src/*.rs`, of the release rule, of
/// the logo rule and of the pattern.
fn store(repo: &Repo) -> [String; 4] {
    let trail = add(repo, &["--", "the logo"]);
    let pick = add(repo, &["--under", &trail[..8], "--", "a sharp logo"]);
    let o = cli(
        repo,
        &["step", "done", &pick[..8], "--as-planned", "--", "sharp"],
    );
    assert!(o.status.success(), "{}", err(&o));
    [
        decide(
            repo,
            &["--path", "src/*.rs"],
            "When Rust code changes: run cargo fmt, because CI checks it.",
        ),
        decide(
            repo,
            &["--situation"],
            "When a release is cut: tag it from main, because the build reads the tag.",
        ),
        decide(
            repo,
            &["--path", "*.svg"],
            "When a logo is drawn: export it as SVG, because the site scales it.",
        ),
        decide(
            repo,
            &["--path", "*.svg", "--good", &pick[..8]],
            "When a logo is drawn: keep sharp corners, because the person picked them.",
        ),
    ]
}

/// `recall` prints the rules that fit an ask by its files and by its words, and leaves out the rest
/// (`spec/02-cli.md`, "Recall").
#[test]
fn recall_prints_the_rules_that_fit_an_ask() {
    let repo = Repo::new();
    let [rust, release, logo, pattern] = store(&repo);
    let said = recall(&repo, &["src/main.rs", "--", "cut the release of the CLI"]);
    assert!(said.contains(&rust[..8]), "{said}");
    assert!(said.contains(&release[..8]), "{said}");
    assert!(!said.contains(&logo[..8]), "{said}");
    assert!(!said.contains(&pattern[..8]), "{said}");
}

/// `recall` prints the rules first, then the patterns with their examples (`spec/02-cli.md`,
/// "Recall").
#[test]
fn recall_prints_the_patterns_after_the_rules() {
    let repo = Repo::new();
    let [_, _, logo, pattern] = store(&repo);
    let said = recall(&repo, &["logo.svg", "--", "draw a new logo"]);
    let lines: Vec<&str> = said.lines().collect();
    let at = |id: &str| lines.iter().position(|l| l.starts_with(&id[..8])).unwrap();
    let rules = lines.iter().position(|l| *l == "rules").unwrap();
    let patterns = lines.iter().position(|l| *l == "patterns").unwrap();
    assert!(rules < at(&logo) && at(&logo) < patterns, "{said}");
    assert!(patterns < at(&pattern), "{said}");
    assert!(
        lines[at(&pattern) + 1].trim_start().starts_with("good"),
        "{said}"
    );
}

/// `recall` says that nothing fits when no rule or pattern does.
#[test]
fn recall_says_when_nothing_fits() {
    let repo = Repo::new();
    store(&repo);
    let said = recall(&repo, &["--", "water the plants"]);
    assert_eq!(said.trim(), "nothing fits this ask");
}

/// `recall` needs the ask after `--`.
#[test]
fn recall_needs_the_ask() {
    let repo = Repo::new();
    let o = cli(&repo, &["recall", "src/main.rs"]);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
}

/// `grep` marks a pattern, and the day `recall` last brought a rule.
#[test]
fn grep_marks_patterns_and_the_day_of_a_recall() {
    let repo = Repo::new();
    let [rust, _, logo, pattern] = store(&repo);
    recall(&repo, &["logo.svg", "--", "draw a new logo"]);
    let o = cli(&repo, &["grep", " rule "]);
    let said = out(&o);
    let line = |id: &str| {
        said.lines()
            .find(|l| l.contains(&format!("rule {}", &id[..8])))
            .unwrap()
            .to_string()
    };
    assert!(line(&pattern).contains(" pattern recalled:"), "{said}");
    assert!(line(&logo).contains(" recalled:"), "{said}");
    assert!(!line(&logo).contains(" pattern"), "{said}");
    assert!(!line(&rust).contains("recalled:"), "{said}");
}
