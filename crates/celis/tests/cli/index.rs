//! CIP-02, "Patterns", and CIP-06, "Uses": the index of patterns, and the steps that use them.

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

/// `step add` accepts `--recalled`, which older plugins give, with any id and on any step, and writes
/// nothing for it (`spec/02-cli.md`, "Step").
#[test]
fn step_add_accepts_recalled_and_writes_nothing_for_it() {
    let repo = Repo::new();
    let a = decide(&repo, &[], "When a logo is drawn: keep it flat.");
    let ask = add(&repo, &["--recalled", &a[..8], "--", "Draw the logo"]);
    add(
        &repo,
        &[
            "--under",
            &ask[..8],
            "--recalled",
            "0000aaaa",
            "--",
            "A part",
        ],
    );
    let steps = labelled(&repo, "step");
    assert_eq!(steps.len(), 2);
    assert!(steps.iter().all(|n| n.tag("recalled").is_none()));
}

/// The `grep` view counts the steps that use a pattern, and gives the day of the newest
/// (`spec/02-cli.md`, "Grep").
#[test]
fn grep_counts_the_uses_and_gives_the_day_of_the_newest() {
    let repo = Repo::new();
    let id = decide(&repo, &[], "When a logo is drawn: keep it flat.");
    let other = decide(&repo, &[], "When a release is cut: tag it.");
    let ask = add(&repo, &["--", "Draw the logo"]);
    add(
        &repo,
        &[
            "--under",
            &ask[..8],
            "--uses",
            &id[..8],
            "--",
            "Draw it flat",
        ],
    );
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
    assert!(
        line(&id).contains(&format!(" uses:1 used:{day} ")),
        "{grep}"
    );
    assert!(!line(&other).contains("uses:"), "{grep}");
    assert!(!line(&other).contains("used:"), "{grep}");
}

/// A reader ignores the `recalled` tag that older writers wrote on an ask (`spec/README.md`,
/// "Names").
#[test]
fn an_old_recalled_tag_is_ignored() {
    let repo = Repo::new();
    let id = decide(&repo, &[], "When a logo is drawn: keep it flat.");
    let mut tags = pattern_tags((&repo.project_id(), "repo"));
    tags.truncate(5);
    tags.extend([
        vec!["recalled".to_string(), id.clone()],
        vec!["L".to_string(), "celis.step".to_string()],
        vec![
            "l".to_string(),
            "step".to_string(),
            "celis.step".to_string(),
        ],
    ]);
    let ask = Note::write(&account(), 1111u16, now(), tags, "Draw the logo").unwrap();
    put(&repo, &ask);
    let grep = out(&cli(&repo, &["grep", &id[..8]]));
    assert!(!grep.contains("uses:"), "{grep}");
    assert!(!grep.contains("used:"), "{grep}");
    let lesson = out(&cli(&repo, &["extract", &ask.id[..8]]));
    assert!(!lesson.contains("  pattern "), "{lesson}");
}

/// A pattern whose first line is `Name: <name>` shows its Name in the index, in `stats`, in the `like`
/// lines and as the heading of `patterns <id>`, and its When is its next line (`spec/02-cli.md`,
/// "Patterns").
#[test]
fn a_named_pattern_shows_its_name() {
    let repo = Repo::new();
    let text = "Name: Light on two sides\nWhen a living room is lit\nForces: one window glares\nDo: give it windows on two sides\nBecause: light from two sides softens shadows";
    let room = decide(&repo, &[], text);
    let index = out(&cli(&repo, &["patterns"]));
    assert_eq!(
        index.trim_end(),
        format!("{} project Light on two sides", &room[..8])
    );
    let whole = out(&cli(&repo, &["patterns", &room[..8]]));
    assert!(whole.starts_with("## Light on two sides\n"), "{whole}");
    assert!(whole.contains("When a living room is lit  \n"), "{whole}");
    let line = out(&cli(&repo, &["stats"]));
    assert!(
        line.lines()
            .next()
            .unwrap()
            .ends_with(" Light on two sides"),
        "{line}"
    );
    let stats = out(&cli(&repo, &["stats", "--json"]));
    let first: serde_json::Value = serde_json::from_str(stats.lines().next().unwrap()).unwrap();
    assert_eq!(first["pattern"]["name"], "Light on two sides", "{stats}");
    assert_eq!(
        first["pattern"]["when"], "When a living room is lit",
        "{stats}"
    );
    let like = cli(
        &repo,
        &[
            "pattern",
            "--",
            "When a living room is lit at night: dim it.",
        ],
    );
    assert!(
        out(&like).contains(&format!("like    {} Light on two sides", &room[..8])),
        "{}",
        out(&like)
    );
}

/// A text whose first line is not a When, and whose next line is, takes that first line as its Name
/// (`spec/02-cli.md`, "Patterns").
#[test]
fn a_bare_first_line_before_a_when_is_the_name() {
    let repo = Repo::new();
    let text = "Light on two sides\nWhen a living room is lit\nForces: one window glares\nDo: add a second window\nBecause: two sides soften shadows";
    let room = decide(&repo, &[], text);
    let index = out(&cli(&repo, &["patterns"]));
    assert_eq!(
        index.trim_end(),
        format!("{} project Light on two sides", &room[..8])
    );
    let tidy = out(&cli(&repo, &["patterns", "--tidy"]));
    assert!(!tidy.contains("nameless"), "{tidy}");
}

/// A pattern has no level until 3 steps use it, `*` then, `**` when its uses are of 2 projects, and no
/// level once it misled; the index and the whole pattern show it (`spec/02-cli.md`, "Levels").
#[test]
fn a_pattern_earns_its_level_from_its_uses() {
    let repo = Repo::new();
    let id = decide(&repo, &["--scope", "me"], "When a price prints: use cents.");
    let other = repo.other("shop");
    let line = |folder: &std::path::Path| {
        let o = repo.run(folder, &["patterns"], &[], "");
        out(&o).lines().next().unwrap().to_string()
    };
    let using = |folder: &std::path::Path, n: usize| {
        let title = format!("Print price {n}");
        let args = ["step", "add", "--new", "--uses", &id[..8], "--", &title];
        let o = repo.run(folder, &args, &[], "");
        assert!(o.status.success(), "{}", err(&o));
    };
    for n in 0..2 {
        using(&repo.path(), n);
    }
    assert_eq!(
        line(&repo.path()),
        format!("{} me When a price prints", &id[..8])
    );
    using(&repo.path(), 2);
    assert_eq!(
        line(&repo.path()),
        format!("{} me * When a price prints", &id[..8])
    );
    using(&other, 3);
    assert_eq!(
        line(&repo.path()),
        format!("{} me ** When a price prints", &id[..8])
    );
    let whole = out(&cli(&repo, &["patterns", &id[..8]]));
    assert!(whole.contains(" · me · **"), "{whole}");
    let ask = add(&repo, &["--new", "--uses", &id[..8], "--", "Print price 4"]);
    support::finish(&repo, &ask[..8]);
    add(
        &repo,
        &["--replaces", &ask[..8], "--", "Print price 4 again"],
    );
    assert_eq!(
        line(&repo.path()),
        format!("{} me When a price prints", &id[..8])
    );
}
