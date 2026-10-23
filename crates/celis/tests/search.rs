//! CIP-02, "Search".

mod support;

use support::{account, cli, err, now, out, refusal, stored_rule, tiny_model, Repo, DAY};

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

/// With no model, `search` prints the `none` line and exits 0 (`spec/02-cli.md`, "Search").
#[test]
fn with_no_model_search_names_grep() {
    let repo = Repo::new();
    let o = cli(&repo, &["search", "--", "prices"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(
        out(&o).trim(),
        "celis search has no model: search with celis grep, or run celis setup."
    );
}

/// `search` refuses empty text, an unknown kind and a limit that is not a positive number
/// (`spec/02-cli.md`, "Search").
#[test]
fn search_refuses_a_bad_text_kind_or_limit() {
    let repo = Repo::new();
    for args in [
        &["search", "--json", "--", " "][..],
        &["search", "--json", "--kind", "proof", "--", "prices"],
        &["search", "--json", "--limit", "0", "--", "prices"],
        &["search", "--json", "--limit", "many", "--", "prices"],
    ] {
        refusal(&cli(&repo, args), 2, "usage");
    }
}

/// `search` gives the records most alike first, at most `--limit`, of the `--kind` asked
/// (`spec/02-cli.md`, "Search").
#[test]
fn search_gives_the_records_most_alike_first() {
    let repo = Repo::new();
    tiny_model(&repo);
    for rule in ["keep prices in cents", "rust release"] {
        assert!(cli(&repo, &["decide", "--", rule]).status.success());
    }
    let step = cli(&repo, &["step", "add", "--new", "--", "release linux"]);
    assert!(step.status.success(), "{}", err(&step));
    let found = objects(&repo, &["--", "keep prices in cents"]);
    assert_eq!(found[0]["content"], "keep prices in cents");
    assert_eq!(found[0]["kind"], "rule");
    assert!((found[0]["score"].as_f64().unwrap() - 1.0).abs() < 1e-4);
    assert!(found.len() >= 3, "{found:?}");
    let steps = objects(&repo, &["--kind", "step", "--", "rust"]);
    assert!(steps.iter().all(|f| f["kind"] == "step"), "{steps:?}");
    assert_eq!(objects(&repo, &["--limit", "1", "--", "rust"]).len(), 1);
    let o = cli(&repo, &["search", "--", "keep prices in cents"]);
    let first = out(&o).lines().next().unwrap().to_string();
    assert!(first.starts_with("1.00 "), "{first}");
    assert!(first.ends_with(" keep prices in cents"), "{first}");
}

/// `search` gives only the live rules in scope in the working folder's project
/// (`spec/02-cli.md`, "Search").
#[test]
fn search_leaves_out_the_rules_of_another_project() {
    let repo = Repo::new();
    tiny_model(&repo);
    stored_rule(
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
    let found = objects(&repo, &["--", "keep prices in cents"]);
    assert!(
        found.iter().all(|f| f["content"] != "keep prices in cents"),
        "{found:?}"
    );
}
