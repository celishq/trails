use super::*;
use celis_nostr::{Account, Note};
use celis_patterns::{PatternRecord, Scope};
use celis_store::{Anchor, PatternId, StepId};
use celis_trails::StepRecord;

fn account() -> Account {
    Account::new(&[1u8; 32]).unwrap()
}

fn anchor() -> Anchor {
    Anchor {
        project: "p".into(),
        name: "repo".into(),
    }
}

/// A step of `text` at `at`, under `under`, that uses `uses`.
fn step(text: &str, at: u64, under: Option<&Note>, uses: &[&str]) -> Note {
    StepRecord {
        anchor: anchor(),
        under: under.map(|n| StepId::new(&n.id)),
        replaces: None,
        uses: uses.iter().map(|u| u.to_string()).collect(),
        recalled: Vec::new(),
        places: Vec::new(),
        check: None,
        session: None,
        content: text.into(),
    }
    .record(&account(), at)
    .unwrap()
}

/// A pattern of `text` at `at` that replaces `replaces`.
fn pattern(text: &str, at: u64, replaces: Option<&Note>) -> Note {
    PatternRecord {
        anchor: anchor(),
        scope: Scope::Project,
        replaces: replaces.map(|n| PatternId::new(&n.id)),
        content: text.into(),
        paths: vec![],
        old_rule: false,
        within: vec![],
        examples: Default::default(),
    }
    .record(&account(), at)
    .unwrap()
}

/// Reuse counts the uses of a live pattern, not of a replaced one or of a step (`spec/06-reuse.md`,
/// "Uses").
#[test]
fn reuse_counts_live_pattern_uses() {
    let old = pattern("old", 1, None);
    let new = pattern("new", 2, Some(&old));
    let trail = step("trail", 10, None, &[]);
    let source = step("source", 11, Some(&trail), &[]);
    let both = step("both", 21, Some(&trail), &[&source.id, &new.id, &old.id]);
    let other = step("other", 22, Some(&trail), &[&new.id]);
    let live = new.id.clone();
    let records = vec![old, new, trail, source, both.clone(), other];
    let steps = Steps::new(&records, 50);
    let patterns = Patterns::new(&records);
    let reuse = Reuse::new(&steps);
    let used = reuse.used(&patterns);
    assert_eq!(used.len(), 1);
    assert_eq!((used[0].0.id(), used[0].1), (live.as_str(), 2));
}

/// Reuse counts the uses of a live pattern by some steps, not of a replaced one or of a step
/// (`spec/06-reuse.md`, "Uses").
#[test]
fn reuse_counts_live_pattern_uses_of_some_steps() {
    let old = pattern("old", 1, None);
    let new = pattern("new", 2, Some(&old));
    let trail = step("trail", 10, None, &[]);
    let source = step("source", 11, Some(&trail), &[]);
    let both = step("both", 21, Some(&trail), &[&source.id, &new.id, &old.id]);
    let other = step("other", 22, Some(&trail), &[&new.id]);
    let records = vec![old, new, trail, source, both.clone(), other];
    let steps = Steps::new(&records, 50);
    let patterns = Patterns::new(&records);
    let reuse = Reuse::new(&steps);
    let picked = vec![steps.get(&both.id).unwrap()];
    assert_eq!(reuse.uses_of(&picked, &patterns), 1);
    assert_eq!(reuse.uses_of(&steps.all(), &patterns), 2);
}
