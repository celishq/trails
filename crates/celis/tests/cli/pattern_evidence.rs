//! CIP-02, "Step": the order of the patterns `celis step done` prints.

use crate::support;

use celis_nostr::Note;
use support::{account, pattern_tags, put, Repo, DAY};

/// The texts of the patterns `celis step done` prints for a step that changed `file`.
fn patterns(repo: &Repo, file: &str) -> Vec<String> {
    let lines = support::matched(repo, &repo.path(), &[file]).into_iter();
    lines
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

/// Patterns come narrowest first, then by id: a pattern made by a correction comes no sooner
/// (`spec/02-cli.md`, "Step").
#[test]
fn patterns_rank_the_narrowest_then_by_id() {
    let repo = Repo::new();
    let at = support::now() - DAY;
    let gone = tagged(&repo, &[("path", "*.py")], "gone", at);
    let mut wide = vec![
        tagged(
            &repo,
            &[("path", "*.py"), ("replaces", &gone.id)],
            "correction",
            at + 1,
        ),
        tagged(&repo, &[("path", "*.py")], "older", at + 2),
        tagged(&repo, &[("path", "*.py")], "wide", at + 4),
    ];
    tagged(&repo, &[("path", "a.py")], "named", at + 3);
    wide.sort_by(|a, b| a.id.cmp(&b.id));
    let mut want = vec!["named".to_string()];
    want.extend(wide.into_iter().map(|n| n.content));
    assert_eq!(patterns(&repo, "a.py"), want);
}
