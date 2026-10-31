use super::*;
use crate::{StepRecord, Steps};
use celis_nostr::Account;
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
        misled: Vec::new(),
        places: Vec::new(),
        check: None,
        session: None,
        agent: None,
        content: "a step".into(),
    }
    .record(&account(), at)
    .unwrap()
}

fn note(text: &str, on: &Note, at: u64) -> Note {
    let mark = Mark::Note { text: text.into() };
    mark.record(&account(), &on.id, at).unwrap()
}

/// A trail's notes are the notes on each step of its tree, oldest first; a note of another trail is
/// none (`spec/04-trails.md`, "Notes").
#[test]
fn a_trails_notes_are_those_on_its_tree() {
    let trail = step(None, 1);
    let ask = step(Some(&trail), 2);
    let other = step(None, 3);
    let on_ask = note("ask", &ask, 10);
    let on_trail = note("trail", &trail, 5);
    let theirs = note("theirs", &other, 11);
    let records = vec![
        trail.clone(),
        ask,
        other,
        on_ask.clone(),
        on_trail.clone(),
        theirs,
    ];
    let (steps, notes) = (Steps::new(&records, 20), Notes::new(&records));
    let ids: Vec<&str> = notes
        .of(steps.get(&trail.id).unwrap())
        .into_iter()
        .map(|n| n.id.as_str())
        .collect();
    assert_eq!(ids, [&on_trail.id, &on_ask.id]);
}
