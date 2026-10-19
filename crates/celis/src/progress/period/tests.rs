use super::*;
use celis_nostr::{Account, Note};
use celis_trails::step::steps::LIFE;
use celis_trails::{Anchor, Mark, Rule, Scope, StepId, StepRecord, Steps};

fn account() -> Account {
    Account::new(&[1u8; 32]).unwrap()
}

/// A step of `text` at `at`, under `under`, that uses `uses` and replaces `replaces`.
fn step(text: &str, at: u64, under: Option<&Note>, uses: &[&str], replaces: Option<&Note>) -> Note {
    StepRecord {
        anchor: Anchor {
            project: "p".into(),
            name: "repo".into(),
        },
        under: under.map(|n| StepId::new(&n.id)),
        replaces: replaces.map(|n| StepId::new(&n.id)),
        uses: uses.iter().map(|u| u.to_string()).collect(),
        places: Vec::new(),
        check: None,
        session: None,
        content: text.into(),
    }
    .record(&account(), at)
    .unwrap()
}

fn done(id: &str, changed: Option<&str>, at: u64) -> Note {
    let mark = Mark::Done {
        proof: "proof".into(),
        changed: changed.map(str::to_string),
    };
    mark.record(&account(), id, at).unwrap()
}

fn dropped(id: &str, at: u64) -> Note {
    let mark = Mark::Drop {
        reason: "why".into(),
    };
    mark.record(&account(), id, at).unwrap()
}

fn titles<'a>(steps: Vec<Step<'a>>) -> Vec<&'a str> {
    steps.into_iter().map(|s| s.title()).collect()
}

/// A period holds the steps done in it, not the trails, and counts those done as planned and those
/// revised later.
#[test]
fn a_period_counts_its_done_steps_as_planned_and_reworked() {
    let trail = step("trail", 10, None, &[], None);
    let early = step("early", 11, Some(&trail), &[], None);
    let planned = step("planned", 12, Some(&trail), &[], None);
    let changed = step("changed", 13, Some(&trail), &[], None);
    let fix = step("fix", 40, Some(&trail), &[], Some(&planned));
    let records = vec![
        trail.clone(),
        early.clone(),
        planned.clone(),
        changed.clone(),
        fix.clone(),
        done(&early.id, None, 15),
        done(&planned.id, None, 25),
        done(&changed.id, Some("other"), 30),
        done(&trail.id, None, 31),
    ];
    let steps = Steps::new(&records, 50);
    let period = Period::new(&steps, 20, 40);
    assert_eq!(titles(period.done()), ["planned", "changed"]);
    assert_eq!(period.planned(), 1);
    assert_eq!(period.reworked(), 1);
}

/// A period counts the steps added in it that use a rule, and not those that use only a step or an
/// id that names no record, and the follows and rule uses of those steps (`spec/06-reuse.md`, "Uses").
#[test]
fn a_period_counts_its_added_steps_that_use_a_rule() {
    let rule = Rule {
        anchor: Anchor {
            project: "p".into(),
            name: "repo".into(),
        },
        scope: Scope::Project,
        replaces: None,
        content: "keep cents".into(),
    }
    .record(&account(), 5)
    .unwrap();
    let trail = step("trail", 10, None, &[], None);
    let source = step("source", 11, Some(&trail), &[], None);
    let ruled = step("ruled", 21, Some(&trail), &[&rule.id], None);
    let follows = step("follows", 22, Some(&trail), &[&source.id], None);
    let dangling = step("dangling", 23, Some(&trail), &["ab12cd34"], None);
    let late = step("late", 45, Some(&trail), &[&rule.id], None);
    let records = vec![rule, trail, source, ruled, follows, dangling, late];
    let steps = Steps::new(&records, 50);
    let period = Period::new(&steps, 20, 40);
    assert_eq!(titles(period.added()), ["ruled", "follows", "dangling"]);
    assert_eq!(period.ruled(), 1);
    assert_eq!(period.follows(), 1);
    assert_eq!(period.uses(), 1);
}

/// A period's closed steps are those done or dropped in it, and those that expired with a trail whose
/// newest record is in it.
#[test]
fn a_period_closes_done_dropped_and_expired_steps() {
    let trail = step("trail", 100, None, &[], None);
    let finished = step("finished", 101, Some(&trail), &[], None);
    let gone = step("gone", 102, Some(&trail), &[], None);
    let left = step("left", 103, Some(&trail), &[], None);
    let records = vec![
        trail,
        finished.clone(),
        gone.clone(),
        left,
        done(&finished.id, None, 110),
        dropped(&gone.id, 120),
    ];
    let steps = Steps::new(&records, 120 + LIFE);
    let period = Period::new(&steps, 100, 200);
    assert_eq!(titles(period.closed()), ["finished", "gone", "left"]);
    assert!(Period::new(&steps, 200, 300).closed().is_empty());
}
