//! CIP-02, "Patterns", and CIP-06, "Recalls": the index of patterns, and the asks that recall them.

use crate::support;

use celis_nostr::Note;
use support::{
    account, add, cli, err, labelled, now, out, pattern_tags, put, refusal, stored_pattern,
    tagged_pattern, Repo, DAY,
};

/// The id `celis pattern <args> -- <text>` prints.
fn decide(repo: &Repo, args: &[&str], text: &str) -> String {
    let mut all = vec!["pattern"];
    all.extend(args);
    all.extend(["--", text]);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().next().unwrap().to_string()
}

/// `search`, which older plugins give, refuses as a usage error that names `celis patterns`, with any
/// flag (`spec/02-cli.md`, "Commands").
#[test]
fn search_refuses_and_names_patterns() {
    let repo = Repo::new();
    for args in [
        &["search", "--", "prices"][..],
        &["search", "src/a.rs", "--kind", "pattern", "--", "prices"],
        &["search", "--limit", "3", "--", "prices"],
    ] {
        let o = cli(&repo, args);
        assert_eq!(o.status.code(), Some(2), "{}", err(&o));
        assert!(err(&o).contains("celis patterns"), "{}", err(&o));
        let mut json = args.to_vec();
        json.insert(1, "--json");
        refusal(&cli(&repo, &json), 2, "usage");
    }
}

/// `patterns` prints each live pattern in scope once, oldest first: its id, scope and When, then its
/// paths and each larger pattern; a four-line pattern's When is its first line, and a one-line
/// pattern's is the part before its first `:` (`spec/02-cli.md`, "Patterns").
#[test]
fn patterns_prints_the_index_oldest_first() {
    let repo = Repo::new();
    let page = decide(&repo, &[], "When a page is drawn: keep one bold thing.");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let hero = decide(
        &repo,
        &["--scope", "me", "--path", "*.svg", "--within", &page],
        "When the hero of a page is drawn\nForces: a big image draws the eye, but hides the text\nDo: show the subject\nBecause: the person reads the subject first",
    );
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let gone = decide(&repo, &[], "When a logo is drawn: keep it round.");
    let logo = decide(
        &repo,
        &["--replaces", &gone[..8]],
        "When a logo is drawn: keep it flat.",
    );
    let said = out(&cli(&repo, &["patterns"]));
    assert_eq!(
        said.lines().collect::<Vec<_>>(),
        [
            format!("{} project When a page is drawn", &page[..8]),
            format!("{} me When the hero of a page is drawn", &hero[..8]),
            "  path   *.svg".to_string(),
            format!("  within {}", &page[..8]),
            format!("{} project When a logo is drawn", &logo[..8]),
        ],
        "{said}"
    );
}

/// `patterns <file>...` prints a `match` line under each pattern for each file that one of its paths
/// matches, a folder as each file in it, and still prints every pattern in the same order
/// (`spec/02-cli.md`, "Patterns").
#[test]
fn patterns_marks_each_pattern_that_a_file_matches() {
    let repo = Repo::new();
    repo.write("shelf/prices.py", "x = 1\n");
    let page = decide(&repo, &[], "When a page is drawn: keep one bold thing.");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let logo = decide(
        &repo,
        &["--path", "*.svg", "--path", "shelf/*.py"],
        "When a logo is drawn: keep it flat.",
    );
    let said = out(&cli(&repo, &["patterns", "logo.svg", "shelf", "notes.md"]));
    assert_eq!(
        said.lines().collect::<Vec<_>>(),
        [
            format!("{} project When a page is drawn", &page[..8]),
            format!("{} project When a logo is drawn", &logo[..8]),
            "  path   *.svg".to_string(),
            "  path   shelf/*.py".to_string(),
            "  match  logo.svg".to_string(),
            "  match  shelf".to_string(),
        ],
        "{said}"
    );
}

/// `patterns` leaves out the patterns of another project (`spec/02-cli.md`, "Patterns").
#[test]
fn patterns_leaves_out_the_patterns_of_another_project() {
    let repo = Repo::new();
    stored_pattern(
        &repo,
        ("0000000000000000000000000000000000000000", "other"),
        &account(),
        (None, None),
        "When prices show: keep cents.",
        now() - DAY,
    );
    let mine = decide(&repo, &[], "When a release is cut: tag it.");
    let said = out(&cli(&repo, &["patterns"]));
    assert_eq!(
        said.trim(),
        format!("{} project When a release is cut", &mine[..8])
    );
}

/// `grep <id8>` gives the whole of a pattern: its four lines on its line, then a line for each example
/// (`spec/02-cli.md`, "Grep").
#[test]
fn grep_gives_the_whole_pattern_with_its_examples() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the logo"]);
    let pick = add(&repo, &["--under", &trail[..8], "--", "a sharp logo"]);
    let o = cli(
        &repo,
        &[
            "step",
            "done",
            &pick[..8],
            "--as-planned",
            "--by",
            "person",
            "--",
            "sharp",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    repo.write("logo.svg", "<svg/>\n");
    let url = "https://example.com/round.png";
    let id = decide(
        &repo,
        &["--good", &pick[..8], "--good", "logo.svg", "--bad", url],
        "When a logo is drawn\nForces: round is soft, but the brand is sharp\nDo: keep sharp corners\nBecause: the person picked them",
    );
    let said = out(&cli(&repo, &["grep", &id[..8]]));
    let short = &id[..8];
    let blob = {
        use sha2::{Digest, Sha256};
        let sha = format!("{:x}", Sha256::digest(b"<svg/>\n"));
        repo.home()
            .join("trails/blobs")
            .join(sha)
            .display()
            .to_string()
    };
    let want = [
        "- patterns of no trail".to_string(),
        format!("  [repo] pattern {short} live {{project}} When a logo is drawn Forces: round is soft, but the brand is sharp Do: keep sharp corners Because: the person picked them"),
        format!("  [repo] good {short} {} a sharp logo · sharp", &pick[..8]),
        format!("  [repo] good {short} logo.svg · {blob}"),
        format!("  [repo] bad {short} {url}"),
    ];
    let lines: Vec<&str> = said.lines().take(want.len()).collect();
    assert_eq!(lines, want, "{said}");
}

/// A pattern with the `binds` label that older writers wrote reads as a plain pattern in the index,
/// and the `grep` view marks it `old-rule` (`spec/05-patterns.md`, "Reading").
#[test]
fn an_old_rule_reads_as_a_plain_pattern() {
    let repo = Repo::new();
    let id = repo.project_id();
    let label = vec![vec![
        "l".to_string(),
        "binds".to_string(),
        "celis.pattern".to_string(),
    ]];
    let old = tagged_pattern(
        &repo,
        (&id, "repo"),
        &account(),
        (None, None),
        label,
        "When a logo is drawn: keep it flat.",
        now() - DAY,
    );
    let said = out(&cli(&repo, &["patterns"]));
    assert_eq!(
        said.trim(),
        format!("{} project When a logo is drawn", &old.id[..8])
    );
    let line = out(&cli(&repo, &["grep", &old.id[..8]]));
    assert!(line.contains(" old-rule"), "{line}");
}

/// A pattern with the deprecated `promoted` label reads as a plain pattern, with no `old-rule` mark
/// (`spec/README.md`, "Names").
#[test]
fn a_promoted_pattern_guides() {
    let repo = Repo::new();
    let id = repo.project_id();
    let label = vec![vec![
        "l".to_string(),
        "promoted".to_string(),
        "celis.decision".to_string(),
    ]];
    let old = tagged_pattern(
        &repo,
        (&id, "repo"),
        &account(),
        (None, None),
        label,
        "When a logo is drawn: keep it flat.",
        now() - DAY,
    );
    let said = out(&cli(&repo, &["patterns"]));
    assert!(said.contains(&old.id[..8]), "{said}");
    let line = out(&cli(&repo, &["grep", &old.id[..8]]));
    assert!(!line.contains(" old-rule"), "{line}");
}

/// `step add --recalled` writes the whole id of each pattern, once, as a `recalled` tag on the ask, a step
/// with no `under` (`spec/06-reuse.md`, "Recalls"; `spec/02-cli.md`, "Step").
#[test]
fn step_add_writes_each_recall_on_the_ask() {
    let repo = Repo::new();
    let a = decide(&repo, &[], "When a logo is drawn: keep it flat.");
    let b = decide(
        &repo,
        &["--scope", "me"],
        "When you commit: use plain words.",
    );
    let ask = add(
        &repo,
        &[
            "--session",
            "s1",
            "--recalled",
            &a[..8],
            "--recalled",
            &b,
            "--recalled",
            &a[..8],
            "--",
            "Draw the logo",
        ],
    );
    let steps = labelled(&repo, "step");
    let recalled = |id: &str| -> Vec<String> {
        let step = steps.iter().find(|n| n.id == id).unwrap();
        let tags = step.tags.iter().filter(|t| t[0] == "recalled");
        tags.map(|t| t[1].clone()).collect()
    };
    assert_eq!(steps.len(), 1);
    assert_eq!(recalled(&ask), [a.clone(), b.clone()]);
    let more = add(&repo, &["--recalled", &b[..8], "--", "Draw the icon"]);
    let steps = labelled(&repo, "step");
    let step = steps.iter().find(|n| n.id == more).unwrap();
    assert_eq!(step.tag("recalled"), Some(b.as_str()));
}

/// `step add` refuses a `--recalled` that no live pattern in scope has with `no-pattern`, and one on a
/// step that is not an ask as a usage error (`spec/02-cli.md`, "Step").
#[test]
fn step_add_refuses_a_recall_of_no_pattern_or_off_an_ask() {
    let repo = Repo::new();
    let a = decide(&repo, &[], "When a logo is drawn: keep it flat.");
    let o = cli(
        &repo,
        &[
            "step",
            "add",
            "--json",
            "--session",
            "s1",
            "--recalled",
            "0000aaaa",
            "--",
            "Draw it",
        ],
    );
    refusal(&o, 1, "no-pattern");
    let ids = add(&repo, &["--session", "s1", "--", "Draw the logo"]);
    let ask = ids.lines().next().unwrap();
    let o = cli(
        &repo,
        &[
            "step",
            "add",
            "--json",
            "--under",
            ask,
            "--recalled",
            &a[..8],
            "--",
            "A part",
        ],
    );
    refusal(&o, 2, "usage");
    assert!(labelled(&repo, "step")
        .iter()
        .all(|n| n.tag("recalled").is_none()));
}

/// `stats` counts the asks that recalled a pattern, and the `grep` view gives the day of the newest
/// (`spec/06-reuse.md`, "Recalls"; `spec/02-cli.md`, "Grep").
#[test]
fn stats_and_grep_count_the_recalls_of_the_asks() {
    let repo = Repo::new();
    let id = decide(&repo, &[], "When a logo is drawn: keep it flat.");
    let other = decide(&repo, &[], "When a release is cut: tag it.");
    let index = out(&cli(&repo, &["patterns"]));
    assert!(index.contains(&id[..8]), "{index}");
    add(
        &repo,
        &[
            "--session",
            "s1",
            "--recalled",
            &id[..8],
            "--",
            "Draw the logo",
        ],
    );
    let stats = out(&cli(&repo, &["stats"]));
    let line = format!(
        "pattern {} When a logo is drawn: keep it flat. · 0 examples · recalled 1",
        &id[..8]
    );
    assert!(stats.lines().any(|l| l == line), "{stats}");
    assert!(!stats.contains(&other[..8]), "{stats}");
    let grep = out(&cli(&repo, &["grep", " pattern "]));
    let made = format!("^[0-9-]* - .repo. pattern {}", &id[..8]);
    let made = out(&cli(&repo, &["grep", "-o", &made]));
    let day = made.split(' ').next().unwrap();
    let line = |id: &str| {
        grep.lines()
            .find(|l| l.contains(&id[..8]))
            .unwrap()
            .to_string()
    };
    assert!(line(&id).contains(&format!(" recalled:{day} ")), "{grep}");
    assert!(!line(&other).contains("recalled:"), "{grep}");
}

/// A reader ignores a `recalled` that names no record (`spec/06-reuse.md`, "Recalls").
#[test]
fn a_recall_of_no_record_is_ignored() {
    let repo = Repo::new();
    let id = decide(&repo, &[], "When a logo is drawn: keep it flat.");
    let trail = add(&repo, &["--", "the logo"]);
    let mut tags = pattern_tags((&repo.project_id(), "repo"));
    tags.truncate(5);
    tags.extend([
        vec!["under".to_string(), trail.clone()],
        vec!["recalled".to_string(), "f".repeat(64)],
        vec!["L".to_string(), "celis.step".to_string()],
        vec![
            "l".to_string(),
            "step".to_string(),
            "celis.step".to_string(),
        ],
    ]);
    put(
        &repo,
        &Note::write(&account(), 1111u16, now(), tags, "Draw it").unwrap(),
    );
    let stats = out(&cli(&repo, &["stats"]));
    assert!(!stats.contains("recalled"), "{stats}");
    let grep = out(&cli(&repo, &["grep", &id[..8]]));
    assert!(!grep.contains("recalled:"), "{grep}");
}
