//! CIP-05, "Graph", and CIP-02, "Search".

mod support;

use std::path::Path;
use support::{account, cli, err, labelled, now, out, refusal, tagged_pattern, Repo, DAY};

/// The whole id `celis decide <args> -- <text>` prints in `folder`.
fn decide(repo: &Repo, folder: &Path, args: &[&str], text: &str) -> String {
    let mut all = vec!["decide"];
    all.extend(args);
    all.extend(["--", text]);
    let o = repo.run(folder, &all, &[], "");
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().next().unwrap().to_string()
}

/// The `within` tags of the stored pattern `id`.
fn within(repo: &Repo, id: &str) -> Vec<String> {
    let notes = labelled(repo, "decision");
    let note = notes.iter().find(|n| n.id == id).unwrap();
    let tags = note.tags.iter().filter(|t| t[0] == "within");
    tags.map(|t| t[1].clone()).collect()
}

/// `decide --within` writes the whole id of the larger pattern, and keeps it when the pattern is
/// replaced (`spec/05-patterns.md`, "Graph").
#[test]
fn decide_writes_the_larger_pattern_and_a_replacement_keeps_it() {
    let repo = Repo::new();
    let page = decide(
        &repo,
        &repo.path(),
        &[],
        "When a page is drawn: keep one bold thing.",
    );
    let hero = decide(
        &repo,
        &repo.path(),
        &["--within", &page[..8]],
        "When the hero is drawn: show the subject.",
    );
    assert_eq!(within(&repo, &hero), std::slice::from_ref(&page));
    let new = decide(
        &repo,
        &repo.path(),
        &["--replaces", &hero[..8]],
        "When the hero is drawn: show the subject at work.",
    );
    assert_eq!(within(&repo, &new), [page]);
}

/// `decide --within` refuses an id that no live pattern has, with `no-decision` (`spec/02-cli.md`,
/// "Commands").
#[test]
fn decide_refuses_an_unknown_larger_pattern() {
    let repo = Repo::new();
    let o = cli(
        &repo,
        &[
            "decide",
            "--json",
            "--within",
            "0000ffff",
            "--",
            "When x: y.",
        ],
    );
    refusal(&o, 1, "no-decision");
}

/// A `within` may name a pattern of another project (`spec/05-patterns.md`, "Graph").
#[test]
fn a_larger_pattern_may_be_of_another_project() {
    let repo = Repo::new();
    let shop = repo.other("shop");
    let page = decide(
        &repo,
        &shop,
        &[],
        "When a page is drawn: keep one bold thing.",
    );
    let hero = decide(
        &repo,
        &repo.path(),
        &["--within", &page],
        "When the hero is drawn: show the subject.",
    );
    assert_eq!(within(&repo, &hero), [page]);
}

/// A reader ignores a `within` that names no record it holds (`spec/05-patterns.md`, "Graph").
#[test]
fn a_within_of_no_record_is_ignored() {
    let repo = Repo::new();
    let id = repo.project_id();
    let tags = vec![vec!["within".to_string(), "f".repeat(64)]];
    let hero = tagged_pattern(
        &repo,
        (&id, "repo"),
        &account(),
        (None, None),
        tags,
        "When the hero is drawn: show the subject.",
        now() - DAY,
    );
    let said = out(&cli(&repo, &["search", "--", "draw the hero"]));
    assert!(said.contains(&hero.id[..8]), "{said}");
    assert!(!said.contains("next"), "{said}");
}

/// Under each pattern, `search` prints its smaller patterns in scope, newest first, at most 5
/// (`spec/02-cli.md`, "Search").
#[test]
fn search_prints_at_most_5_smaller_patterns_under_a_pattern() {
    let repo = Repo::new();
    let page = decide(
        &repo,
        &repo.path(),
        &[],
        "When a page is drawn: keep one bold thing.",
    );
    let id = repo.project_id();
    let smaller: Vec<String> = (0..6)
        .map(|n| {
            let tags = vec![vec!["within".to_string(), page.clone()]];
            let note = tagged_pattern(
                &repo,
                (&id, "repo"),
                &account(),
                (None, None),
                tags,
                &format!("When part {n} is drawn: keep it quiet."),
                now() - DAY + n,
            );
            note.id
        })
        .collect();
    let said = out(&cli(
        &repo,
        &["search", "--kind", "pattern", "--", "draw a page"],
    ));
    let next: Vec<&str> = said
        .lines()
        .filter(|l| l.trim_start().starts_with("next"))
        .collect();
    let want: Vec<String> = smaller
        .iter()
        .rev()
        .take(5)
        .enumerate()
        .map(|(n, id)| {
            format!(
                "  next    {} When part {} is drawn: keep it quiet.",
                &id[..8],
                5 - n
            )
        })
        .collect();
    assert_eq!(next, want, "{said}");
}

/// `patterns --markdown` names each smaller pattern under the larger one (`spec/02-cli.md`, "Patterns").
#[test]
fn the_markdown_page_names_the_smaller_patterns() {
    let repo = Repo::new();
    let page = decide(
        &repo,
        &repo.path(),
        &[],
        "When a page is drawn: keep one bold thing.",
    );
    let hero = decide(
        &repo,
        &repo.path(),
        &["--within", &page[..8]],
        "When the hero is drawn: show the subject.",
    );
    let said = out(&cli(&repo, &["patterns", "--markdown"]));
    let line = format!(
        "- next: `{}` When the hero is drawn: show the subject.",
        &hero[..8]
    );
    let at = said.find(&line).unwrap_or_else(|| panic!("{said}"));
    assert!(said.find("## When a page is drawn").unwrap() < at, "{said}");
}
