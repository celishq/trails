//! CIP-02, "Search".

mod support;

use support::{
    account, add, cli, err, now, out, refusal, stored_pattern, tagged_pattern, tiny_model, Repo,
    DAY,
};

/// The JSON objects `celis search --json <args>` prints, one a line.
fn objects(repo: &Repo, args: &[&str]) -> Vec<serde_json::Value> {
    let mut all = vec!["search", "--json"];
    all.extend(args);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o)
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// With no model, `search --kind step` prints the `no model` line and exits 0 (`spec/02-cli.md`,
/// "Search").
#[test]
fn with_no_model_search_names_grep() {
    let repo = Repo::new();
    let o = cli(&repo, &["search", "--kind", "step", "--", "prices"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(
        out(&o).trim(),
        "celis search has no model: search with celis grep, or run celis setup."
    );
}

/// `search` refuses empty text, an unknown kind, a limit that is not a positive number, and `--json`
/// without `--kind trail` or `--kind step` (`spec/02-cli.md`, "Search").
#[test]
fn search_refuses_a_bad_text_kind_or_limit() {
    let repo = Repo::new();
    for args in [
        &["search", "--json", "--", " "][..],
        &["search", "--json", "--kind", "proof", "--", "prices"],
        &["search", "--json", "--limit", "0", "--", "prices"],
        &["search", "--json", "--limit", "many", "--", "prices"],
        &["search", "--json", "--", "prices"],
        &["search", "--json", "--kind", "pattern", "--", "prices"],
    ] {
        refusal(&cli(&repo, args), 2, "usage");
    }
}

/// With `--kind step` or `--kind trail`, `search` gives the records of that kind most alike first, at
/// most `--limit` (`spec/02-cli.md`, "Search").
#[test]
fn search_gives_the_records_most_alike_first() {
    let repo = Repo::new();
    tiny_model(&repo);
    let trail = trail(&repo);
    for title in ["keep prices in cents", "release linux"] {
        add(&repo, &["--under", &trail[..8], "--", title]);
    }
    let found = objects(&repo, &["--kind", "step", "--", "keep prices in cents"]);
    assert_eq!(found[0]["content"], "keep prices in cents");
    assert!(found.iter().all(|f| f["kind"] == "step"), "{found:?}");
    assert!((found[0]["score"].as_f64().unwrap() - 1.0).abs() < 1e-4);
    let trails = objects(&repo, &["--kind", "trail", "--", "rust"]);
    assert!(trails.iter().all(|f| f["kind"] == "trail"), "{trails:?}");
    let one = objects(&repo, &["--kind", "trail", "--limit", "1", "--", "rust"]);
    assert_eq!(one.len(), 1);
    let o = cli(
        &repo,
        &["search", "--kind", "step", "--", "keep prices in cents"],
    );
    let first = out(&o).lines().next().unwrap().to_string();
    assert!(first.starts_with("1.00 "), "{first}");
    assert!(first.ends_with(" keep prices in cents"), "{first}");
}

/// `search` gives only the live patterns in scope in the working folder's project
/// (`spec/02-cli.md`, "Search").
#[test]
fn search_leaves_out_the_patterns_of_another_project() {
    let repo = Repo::new();
    tiny_model(&repo);
    stored_pattern(
        &repo,
        ("0000000000000000000000000000000000000000", "other"),
        &account(),
        (None, None),
        "keep prices in cents",
        now() - DAY,
    );
    assert!(cli(&repo, &["decide", "--", "rust release"])
        .status
        .success());
    let said = search(&repo, &["--", "keep prices in cents"]);
    assert!(!said.contains("keep prices in cents"), "{said}");
}

/// The id `celis decide <args> -- <text>` prints.
fn decide(repo: &Repo, args: &[&str], text: &str) -> String {
    let mut all = vec!["decide"];
    all.extend(args);
    all.extend(["--", text]);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().next().unwrap().to_string()
}

/// The id of a new trail.
fn trail(repo: &Repo) -> String {
    let ids = add(repo, &["--", "the shop"]);
    ids.lines().next().unwrap().to_string()
}

/// What `celis search <args>` prints.
fn search(repo: &Repo, args: &[&str]) -> String {
    let mut all = vec!["search"];
    all.extend(args);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o)
}

/// Closes the step `id` with `proof`, as planned or with `changed`.
fn done(repo: &Repo, id: &str, changed: Option<&str>, proof: &str) {
    let mut all = vec!["step", "done", &id[..8]];
    match changed {
        Some(changed) => all.extend(["--changed", changed]),
        None => all.push("--as-planned"),
    }
    all.extend(["--", proof]);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
}

/// A store of 3 rules and 1 pattern, and the ids of the rule of `src/*.rs`, of the release rule, of the
/// logo rule and of the pattern.
fn store(repo: &Repo) -> [String; 4] {
    let trail = add(repo, &["--", "the logo"]);
    let pick = add(repo, &["--under", &trail[..8], "--", "a sharp logo"]);
    done(repo, &pick, None, "sharp");
    [
        decide(
            repo,
            &["--path", "src/*.rs", "--binds"],
            "When Rust code changes: run cargo fmt, because CI checks it.",
        ),
        decide(
            repo,
            &["--binds"],
            "When a release is cut: tag it from main, because the build reads the tag.",
        ),
        decide(
            repo,
            &["--path", "*.svg", "--binds"],
            "When a logo is drawn: export it as SVG, because the site scales it.",
        ),
        decide(
            repo,
            &["--path", "*.svg", "--good", &pick[..8]],
            "When a logo is drawn: keep sharp corners, because the person picked them.",
        ),
    ]
}

/// `search` prints the patterns that fit an ask by its files and by its words, and leaves out the rest
/// (`spec/02-cli.md`, "Search").
#[test]
fn search_prints_the_patterns_that_fit_an_ask() {
    let repo = Repo::new();
    let [rust, release, logo, pattern] = store(&repo);
    let said = search(&repo, &["src/main.rs", "--", "cut the release of the CLI"]);
    assert!(said.contains(&rust[..8]), "{said}");
    assert!(said.contains(&release[..8]), "{said}");
    assert!(!said.contains(&logo[..8]), "{said}");
    assert!(!said.contains(&pattern[..8]), "{said}");
}

/// `search` finds a pattern by its meaning when it shares no file and no word with the ask
/// (`spec/02-cli.md`, "Search").
#[test]
fn search_finds_a_pattern_by_its_meaning() {
    let repo = Repo::new();
    tiny_model(&repo);
    let pattern = decide(
        &repo,
        &[],
        "When money moves: keep prices in cents, because floats lose a cent.",
    );
    let said = search(&repo, &["--", "keep prices in cents"]);
    assert!(said.contains(&pattern[..8]), "{said}");
}

/// `search` prints the rules first, then the other patterns with their examples (`spec/02-cli.md`,
/// "Search").
#[test]
fn search_prints_the_rules_before_the_other_patterns() {
    let repo = Repo::new();
    let [_, _, logo, pattern] = store(&repo);
    let said = search(&repo, &["logo.svg", "--", "draw a new logo"]);
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

/// A pattern with the deprecated `promoted` label guides, and is no rule (`spec/05-patterns.md`, "Rules").
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
    let rules = search(&repo, &["--kind", "rule", "--", "draw a logo"]);
    assert!(!rules.contains(&old.id[..8]), "{rules}");
    let line = out(&cli(&repo, &["grep", &old.id[..8]]));
    assert!(!line.contains(" binds"), "{line}");
}

/// `search` prints a pattern's first line after its id and scope, and each next line of its text under
/// it (`spec/02-cli.md`, "Search").
#[test]
fn search_prints_each_line_of_a_pattern() {
    let repo = Repo::new();
    let text = "When a logo is drawn\nForces: bold reads far, but thin reads light\nDo: keep it bold\nBecause: the person picked bold";
    let id = decide(&repo, &[], text);
    let said = search(&repo, &["--kind", "pattern", "--", "draw a logo"]);
    let lines: Vec<&str> = said.lines().collect();
    assert_eq!(
        lines,
        [
            "patterns".to_string(),
            format!("{} project When a logo is drawn", &id[..8]),
            "  Forces: bold reads far, but thin reads light".to_string(),
            "  Do: keep it bold".to_string(),
            "  Because: the person picked bold".to_string(),
        ],
        "{said}"
    );
}

/// With `--kind rule` or `--kind pattern`, `search` prints only that part (`spec/02-cli.md`,
/// "Search").
#[test]
fn search_prints_only_the_kind_asked() {
    let repo = Repo::new();
    let [_, _, logo, pattern] = store(&repo);
    let rules = search(&repo, &["logo.svg", "--kind", "rule", "--", "draw a logo"]);
    assert!(
        rules.contains(&logo[..8]) && !rules.contains(&pattern[..8]),
        "{rules}"
    );
    assert!(!rules.contains("patterns"), "{rules}");
    let patterns = search(
        &repo,
        &["logo.svg", "--kind", "pattern", "--", "draw a logo"],
    );
    assert!(patterns.contains(&pattern[..8]), "{patterns}");
    assert!(!patterns.contains(&logo[..8]), "{patterns}");
}

/// `search` prints the done asks like the ask, at most 2, each with the steps right under it and
/// their `changed` lines, and leaves out an open ask (`spec/02-cli.md`, "Search").
#[test]
fn search_prints_the_past_asks_like_the_ask() {
    let repo = Repo::new();
    let trail = trail(&repo);
    let ask = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--",
            "Round the shelf prices to cents",
        ],
    );
    let step = add(
        &repo,
        &["--under", &ask[..8], "--", "Round each price in the shelf"],
    );
    done(
        &repo,
        &step,
        Some("kept floats in the tests"),
        "pytest passes",
    );
    done(&repo, &ask, None, "the shelf shows cents");
    let open = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--",
            "Round the cart prices to cents",
        ],
    );
    let said = search(&repo, &["--", "round the prices of the menu to cents"]);
    let lines: Vec<&str> = said.lines().collect();
    let past = lines.iter().position(|l| *l == "past").unwrap();
    assert!(lines[past + 1].starts_with(&ask[..8]), "{said}");
    assert!(
        lines[past + 2].starts_with(&format!("  step {} done", &step[..8])),
        "{said}"
    );
    assert_eq!(
        lines[past + 3],
        "    changed kept floats in the tests",
        "{said}"
    );
    assert!(!said.contains(&open[..8]), "{said}");
}

/// A past ask with no step under it prints as its own step, with its `changed` line
/// (`spec/02-cli.md`, "Search").
#[test]
fn search_prints_a_past_ask_with_no_steps_as_its_own_step() {
    let repo = Repo::new();
    let trail = trail(&repo);
    let ask = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--",
            "Round the shelf prices to cents",
        ],
    );
    done(
        &repo,
        &ask,
        Some("kept floats in the tests"),
        "pytest passes",
    );
    let said = search(&repo, &["--", "round the prices of the menu to cents"]);
    let lines: Vec<&str> = said.lines().collect();
    let past = lines.iter().position(|l| *l == "past").unwrap();
    let step = format!("  step {} done Round the shelf prices to cents", &ask[..8]);
    assert_eq!(lines[past + 2], step, "{said}");
    assert_eq!(
        lines[past + 3],
        "    changed kept floats in the tests",
        "{said}"
    );
}

/// A past ask is like the ask by the words of its title and of its trail's title (`spec/02-cli.md`,
/// "Search").
#[test]
fn search_reads_the_title_of_a_past_asks_trail() {
    let repo = Repo::new();
    let ids = add(&repo, &["--", "Add vendors_csv to report.py"]);
    let trail = ids.lines().next().unwrap().to_string();
    let ask = add(
        &repo,
        &["--under", &trail[..8], "--", "Write it with a test"],
    );
    done(&repo, &ask, None, "pytest passes");
    let said = search(&repo, &["--", "Add clients_csv to report.py with a test"]);
    assert!(said.contains(&ask[..8]), "{said}");
}

/// `search` prints at most 2 past asks, and with `--limit 1` at most 1 (`spec/02-cli.md`, "Search").
#[test]
fn search_prints_at_most_two_past_asks() {
    let repo = Repo::new();
    let trail = trail(&repo);
    for n in ["one", "two", "three"] {
        let title = format!("Round the {n} prices to cents");
        let ask = add(&repo, &["--under", &trail[..8], "--", &title]);
        done(&repo, &ask, None, "cents");
    }
    let count = |args: &[&str]| {
        let said = search(&repo, args);
        said.lines().filter(|l| l.contains(" · ")).count()
    };
    assert_eq!(count(&["--", "round the prices to cents"]), 2);
    assert_eq!(
        count(&["--limit", "1", "--", "round the prices to cents"]),
        1
    );
}

/// `search` says that nothing fits when no rule, pattern or past ask does.
#[test]
fn search_says_when_nothing_fits() {
    let repo = Repo::new();
    store(&repo);
    let said = search(&repo, &["--", "water the plants"]);
    assert_eq!(said.trim(), "nothing fits this ask");
}

/// `search` needs the ask after `--`.
#[test]
fn search_needs_the_ask() {
    let repo = Repo::new();
    let o = cli(&repo, &["search", "src/main.rs"]);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
}

/// `grep` marks a rule, and the day `search` last brought a pattern.
#[test]
fn grep_marks_patterns_and_the_day_of_a_search() {
    let repo = Repo::new();
    let [rust, _, logo, pattern] = store(&repo);
    search(&repo, &["logo.svg", "--", "draw a new logo"]);
    let o = cli(&repo, &["grep", " pattern "]);
    let said = out(&o);
    let line = |id: &str| {
        said.lines()
            .find(|l| l.contains(&format!("pattern {}", &id[..8])))
            .unwrap()
            .to_string()
    };
    assert!(line(&logo).contains(" binds recalled:"), "{said}");
    assert!(line(&pattern).contains(" recalled:"), "{said}");
    assert!(!line(&pattern).contains(" binds"), "{said}");
    assert!(!line(&rust).contains("recalled:"), "{said}");
}
