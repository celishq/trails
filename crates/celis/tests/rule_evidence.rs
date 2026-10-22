//! CIP-02, "Rules": the rank of the rules `celis rules` prints.

mod support;

use celis_nostr::Note;
use support::{account, err, out, put, rule_tags, Repo, DAY};

/// The texts of the rules `celis rules <file>` gives.
fn rules(repo: &Repo, file: &str) -> Vec<String> {
    let o = repo.run(&repo.path(), &["rules", file], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let shown = out(&o);
    let rules = shown.lines().filter(|l| !l.starts_with("celis: "));
    rules
        .map(|l| l.splitn(3, ' ').nth(2).unwrap().to_string())
        .collect()
}

/// A rule of this repo with the extra tags `tags`, saying `content`, made at `at`, put in the store.
fn tagged(repo: &Repo, tags: &[(&str, &str)], content: &str, at: u64) -> Note {
    let mut all = rule_tags((&repo.project_id(), "repo"));
    all.extend(tags.iter().map(|(n, v)| vec![n.to_string(), v.to_string()]));
    let note = Note::write(&account(), 1111u16, at, all, content).unwrap();
    put(repo, &note);
    note
}

/// A rule made by a correction comes first, then the rest narrowest first, then newest
/// (`spec/02-cli.md`, "Rules").
#[test]
fn rules_rank_a_correction_then_the_narrowest() {
    let repo = Repo::new();
    let at = support::now() - DAY;
    let gone = tagged(&repo, &[("path", "*.py")], "gone", at);
    tagged(
        &repo,
        &[("path", "*.py"), ("replaces", &gone.id)],
        "correction",
        at + 1,
    );
    tagged(&repo, &[("path", "*.py")], "older", at + 2);
    tagged(&repo, &[("path", "a.py")], "named", at + 3);
    tagged(&repo, &[("path", "*.py")], "wide", at + 4);
    assert_eq!(
        rules(&repo, "a.py"),
        ["correction", "named", "wide", "older"]
    );
}
