use super::*;
use celis_nostr::Account;
use celis_trails::{Anchor, RuleId, Scope, StepId, StepRecord};

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
        places: Vec::new(),
        check: None,
        session: None,
        content: text.into(),
    }
    .record(&account(), at)
    .unwrap()
}

/// A rule of `text` at `at` that replaces `replaces`.
fn rule(text: &str, at: u64, replaces: Option<&Note>) -> Note {
    Rule {
        anchor: anchor(),
        scope: Scope::Project,
        replaces: replaces.map(|n| RuleId::new(&n.id)),
        content: text.into(),
    }
    .record(&account(), at)
    .unwrap()
}

/// Reuse counts the follows, and the uses of a live rule, not of a replaced one (`spec/06-reuse.md`,
/// "Uses").
#[test]
fn reuse_counts_the_follows_and_live_rule_uses() {
    let old = rule("old", 1, None);
    let new = rule("new", 2, Some(&old));
    let trail = step("trail", 10, None, &[]);
    let source = step("source", 11, Some(&trail), &[]);
    let both = step("both", 21, Some(&trail), &[&source.id, &new.id, &old.id]);
    let other = step("other", 22, Some(&trail), &[&new.id]);
    let live = new.id.clone();
    let records = vec![old, new, trail, source, both.clone(), other];
    let steps = Steps::new(&records, 50);
    let rules = Rules::new(&records);
    let reuse = Reuse::new(&steps);
    let used = reuse.used(&rules);
    assert_eq!(reuse.total(), 1);
    assert_eq!(used.len(), 1);
    assert_eq!((used[0].0.id.as_str(), used[0].2), (live.as_str(), 2));
}
