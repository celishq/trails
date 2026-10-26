use super::*;
use crate::{StepRecord, Steps};
use celis_nostr::Account;
use celis_patterns::Patterns;
use celis_store::{Anchor, StepId};

fn account() -> Account {
    Account::new(&[1u8; 32]).unwrap()
}

fn step(under: Option<&Note>, at: u64) -> Note {
    StepRecord {
        anchor: Anchor {
            project: "p".into(),
            name: "repo".into(),
        },
        under: under.map(|u| StepId::new(&u.id)),
        replaces: None,
        uses: Vec::new(),
        recalled: Vec::new(),
        places: Vec::new(),
        check: None,
        session: None,
        content: "a step".into(),
    }
    .record(&account(), at)
    .unwrap()
}

/// A pattern of the scope `trail:<trail>` that older writers wrote, replacing `replaces` when given.
fn old(trail: &str, replaces: Option<&Note>, at: u64) -> Note {
    let mut tags: Vec<Vec<String>> = vec![
        vec!["I".into(), "project:p".into()],
        vec!["K".into(), "project".into()],
        vec!["i".into(), "project:p".into()],
        vec!["k".into(), "project".into()],
        vec!["project".into(), "repo".into()],
        vec!["scope".into(), format!("trail:{trail}")],
        vec!["L".into(), "celis.decision".into()],
        vec!["l".into(), "decision".into(), "celis.decision".into()],
    ];
    tags.extend(replaces.map(|r| vec!["replaces".into(), r.id.clone()]));
    Note::write(&account(), 1111u16, at, tags, "When a vendor prints").unwrap()
}

fn mark(mark: Mark, on: &Note, at: u64) -> Note {
    mark.record(&account(), &on.id, at).unwrap()
}

/// A trail's notes are the notes on each step of its tree and each live old pattern of its scope, oldest
/// first; a replaced or dropped old pattern is none (`spec/04-trails.md`, "Notes").
#[test]
fn a_trails_notes_are_those_on_its_tree_and_its_live_old_patterns() {
    let trail = step(None, 1);
    let ask = step(Some(&trail), 2);
    let other = step(None, 3);
    let on_ask = mark(Mark::Note { text: "ask".into() }, &ask, 10);
    let kept = old(&trail.id, None, 5);
    let replaced = old(&trail.id, None, 6);
    let newer = old(&trail.id, Some(&replaced), 7);
    let dropped = old(&trail.id, None, 8);
    let drop = mark(
        Mark::Drop {
            reason: "no".into(),
        },
        &dropped,
        9,
    );
    let theirs = mark(
        Mark::Note {
            text: "theirs".into(),
        },
        &other,
        11,
    );
    let records = vec![
        trail.clone(),
        ask,
        other,
        on_ask.clone(),
        kept.clone(),
        replaced,
        newer.clone(),
        dropped,
        drop,
        theirs,
    ];
    let (steps, notes) = (Steps::new(&records, 20), Notes::new(&records));
    let ids: Vec<&str> = notes
        .of(steps.get(&trail.id).unwrap())
        .into_iter()
        .map(|n| n.id.as_str())
        .collect();
    assert_eq!(ids, [&kept.id, &newer.id, &on_ask.id]);
}

/// A reader reads an old `trail:<id>` pattern as a note, and not as a pattern (`spec/05-patterns.md`,
/// "Reading").
#[test]
fn an_old_trail_pattern_is_no_pattern() {
    let trail = step(None, 1);
    let records = vec![trail.clone(), old(&trail.id, None, 2)];
    assert!(Patterns::new(&records).every().is_empty());
    assert_eq!(Notes::new(&records).on(&trail.id).len(), 1);
}
