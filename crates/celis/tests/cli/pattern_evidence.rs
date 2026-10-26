//! CIP-02, "Search": the rank of the patterns `celis search` prints.

use crate::support;

use celis_nostr::Note;
use support::{account, err, out, pattern_tags, put, Repo, DAY};

/// The texts of the patterns `celis search <file> --kind pattern` gives for an ask that shares no word with
/// a pattern.
fn patterns(repo: &Repo, file: &str) -> Vec<String> {
    let args = ["search", file, "--kind", "pattern", "--", "x"];
    let o = repo.run(&repo.path(), &args, &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let shown = out(&o);
    let patterns = shown
        .lines()
        .filter(|l| !l.starts_with(' ') && *l != "patterns");
    patterns
        .map(|l| l.splitn(3, ' ').nth(2).unwrap().to_string())
        .collect()
}

/// A pattern of this repo with the extra tags `tags`, saying `content`, made at `at`, put in the store.
fn tagged(repo: &Repo, tags: &[(&str, &str)], content: &str, at: u64) -> Note {
    let mut all = pattern_tags((&repo.project_id(), "repo"));
    all.extend(tags.iter().map(|(n, v)| vec![n.to_string(), v.to_string()]));
    let note = Note::write(&account(), 1111u16, at, all, content).unwrap();
    put(repo, &note);
    note
}

/// Patterns rank narrowest first, then within each a pattern made by a correction first, then newest
/// (`spec/02-cli.md`, "Search").
#[test]
fn patterns_rank_the_narrowest_then_a_correction() {
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
        patterns(&repo, "a.py"),
        ["named", "correction", "wide", "older"]
    );
}
