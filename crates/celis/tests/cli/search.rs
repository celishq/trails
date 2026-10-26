//! CIP-02, "Search".

use crate::support;

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
        &["search", "--json", "--kind", "rule", "--", "prices"],
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
    assert!(cli(&repo, &["pattern", "--", "rust release"])
        .status
        .success());
    let said = search(&repo, &["--", "keep prices in cents"]);
    assert!(!said.contains("keep prices in cents"), "{said}");
}

/// The id `celis pattern <args> -- <text>` prints.
fn decide(repo: &Repo, args: &[&str], text: &str) -> String {
    let mut all = vec!["pattern"];
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

/// A store of 4 patterns, and the ids of the pattern of `src/*.rs`, of the release pattern, of the logo
/// export pattern and of the logo pattern with a good example.
fn store(repo: &Repo) -> [String; 4] {
    let trail = add(repo, &["--", "the logo"]);
    let pick = add(repo, &["--under", &trail[..8], "--", "a sharp logo"]);
    done(repo, &pick, None, "sharp");
    [
        decide(
            repo,
            &["--path", "src/*.rs"],
            "When Rust code changes: run cargo fmt, because CI checks it.",
        ),
        decide(
            repo,
            &[],
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

/// `search` prints the patterns in one part, each with its examples (`spec/02-cli.md`, "Search").
#[test]
fn search_prints_one_part_of_patterns() {
    let repo = Repo::new();
    let [_, _, logo, pattern] = store(&repo);
    let said = search(&repo, &["logo.svg", "--", "draw a new logo"]);
    let lines: Vec<&str> = said.lines().collect();
    let at = |id: &str| lines.iter().position(|l| l.starts_with(&id[..8])).unwrap();
    assert_eq!(lines[0], "patterns", "{said}");
    assert!(!lines.contains(&"rules"), "{said}");
    assert!(at(&logo) > 0 && at(&pattern) > 0, "{said}");
    assert!(
        lines[at(&pattern) + 1].trim_start().starts_with("good"),
        "{said}"
    );
}

/// A pattern with the `binds` label that older writers wrote reads as a plain pattern in `search`, and
/// the `grep` view marks it `old-rule` (`spec/05-patterns.md`, "Reading").
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
    let said = search(&repo, &["--", "draw a logo"]);
    let lines: Vec<&str> = said.lines().collect();
    assert_eq!(lines[0], "patterns", "{said}");
    assert!(lines[1].starts_with(&old.id[..8]), "{said}");
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
    let said = search(&repo, &["--", "draw a logo"]);
    assert!(said.contains(&old.id[..8]), "{said}");
    let line = out(&cli(&repo, &["grep", &old.id[..8]]));
    assert!(!line.contains(" old-rule"), "{line}");
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

/// With `--kind pattern`, `search` prints only that part (`spec/02-cli.md`, "Search").
#[test]
fn search_prints_only_the_kind_asked() {
    let repo = Repo::new();
    let [_, _, _, pattern] = store(&repo);
    let patterns = search(
        &repo,
        &["logo.svg", "--kind", "pattern", "--", "draw a logo"],
    );
    assert!(patterns.contains(&pattern[..8]), "{patterns}");
    assert!(!patterns.contains("past"), "{patterns}");
}

/// `search` prints the done asks like the ask, at most 2, each with the `changed` lines of the steps
/// right under it and no other step line, and leaves out an open ask (`spec/02-cli.md`, "Search").
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
    let plain = add(&repo, &["--under", &ask[..8], "--", "Show the cents"]);
    done(&repo, &plain, None, "the shelf shows cents");
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
    let changed = format!("  changed {} kept floats in the tests", &step[..8]);
    assert_eq!(lines[past + 2], changed, "{said}");
    assert!(!said.contains(&plain[..8]), "{said}");
    assert!(!said.contains("  step "), "{said}");
    assert!(!said.contains(&open[..8]), "{said}");
}

/// A past ask with no step under it prints its own `changed` line (`spec/02-cli.md`, "Search").
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
    let changed = format!("  changed {} kept floats in the tests", &ask[..8]);
    assert_eq!(lines[past + 2], changed, "{said}");
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

/// `search` says that nothing fits when no pattern or past ask does.
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

/// `grep` marks the day `search` last brought a pattern.
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
    assert!(line(&logo).contains(" recalled:"), "{said}");
    assert!(line(&pattern).contains(" recalled:"), "{said}");
    assert!(!line(&rust).contains("recalled:"), "{said}");
}

/// The lines of `said` that start with a pattern's id, in their order.
fn heads<'a>(said: &'a str, ids: &[String]) -> Vec<&'a str> {
    let head = |l: &&str| ids.iter().any(|id| l.starts_with(&id[..8]));
    said.lines().filter(head).collect()
}

/// `search` prints the first 8 patterns, or the first `--limit`, then the `more` line, and only a
/// printed pattern counts as recalled (`spec/02-cli.md`, "Search").
#[test]
fn search_prints_the_first_eight_patterns_and_the_more_line() {
    let repo = Repo::new();
    let patterns: Vec<String> = (0..10)
        .map(|n| {
            let text = format!("When Rust code {n} changes: keep it short.");
            decide(&repo, &["--path", "src/*.rs"], &text)
        })
        .collect();
    let said = search(&repo, &["src/main.rs", "--", "tidy the code"]);
    let shown = heads(&said, &patterns);
    assert_eq!(shown.len(), 8, "{said}");
    let more = "2 more: celis search src/main.rs --kind pattern --limit 10 -- \"tidy the code\"";
    assert_eq!(said.lines().last(), Some(more), "{said}");
    let recalled = out(&cli(&repo, &["grep", " recalled:"]));
    for (n, id) in patterns.iter().enumerate() {
        let marked = recalled.contains(&id[..8]);
        assert_eq!(
            marked,
            shown.iter().any(|l| l.starts_with(&id[..8])),
            "{n} {recalled}"
        );
    }
    let said = search(
        &repo,
        &["src/main.rs", "--limit", "3", "--", "tidy the code"],
    );
    assert_eq!(heads(&said, &patterns).len(), 3, "{said}");
    assert!(
        said.contains("\n7 more: celis search src/main.rs --kind pattern --limit 10 -- "),
        "{said}"
    );
}

/// A pattern in a cell that fits ranks under one that matches a file and over one that fits by words
/// (`spec/02-cli.md`, "Search").
#[test]
fn a_cell_fit_ranks_between_a_path_and_a_word() {
    let repo = Repo::new();
    let path = decide(
        &repo,
        &["--path", "*.svg", "--cell", "brand"],
        "When an icon ships: keep it flat.",
    );
    let named = decide(
        &repo,
        &["--cell", "logo"],
        "When a mark ships: keep two colours.",
    );
    let filed = decide(
        &repo,
        &["--cell", "brand"],
        "When a font ships: keep one family.",
    );
    let word = decide(&repo, &[], "When prices show: keep cents.");
    let ids = [path.clone(), named.clone(), filed.clone(), word.clone()];
    let said = search(&repo, &["logo.svg", "--", "draw a logo with the prices"]);
    let shown = heads(&said, &ids);
    let at = |id: &str| shown.iter().position(|l| l.starts_with(&id[..8])).unwrap();
    assert!(at(&path) < at(&named) && at(&path) < at(&filed), "{said}");
    assert!(at(&named) < at(&word) && at(&filed) < at(&word), "{said}");
}

/// A word in the first line of more than 5% of the live patterns, and of more than 5, does not fit
/// (`spec/02-cli.md`, "Search").
#[test]
fn a_word_of_many_patterns_does_not_fit() {
    let repo = Repo::new();
    let common: Vec<String> = ["runs", "prints", "reads", "writes", "starts", "stops"]
        .iter()
        .map(|verb| decide(&repo, &[], &format!("When celis {verb}: keep it quiet.")))
        .collect();
    let prices = decide(&repo, &[], "When prices show: keep cents.");
    let said = search(&repo, &["--", "celis prices"]);
    assert!(heads(&said, &common).is_empty(), "{said}");
    assert!(said.contains(&prices[..8]), "{said}");
}
