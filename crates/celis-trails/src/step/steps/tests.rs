use super::*;
use crate::{Anchor, State, StepId};
use celis_nostr::Account;

fn account() -> Account {
    Account::new(&[1u8; 32]).unwrap()
}

fn step(text: &str, at: u64) -> Note {
    StepRecord {
        anchor: Anchor {
            project: "p".into(),
            name: "repo".into(),
        },
        under: None,
        replaces: None,
        uses: Vec::new(),
        places: Vec::new(),
        check: None,
        session: None,
        content: text.into(),
    }
    .record(&account(), at)
    .unwrap()
}

fn done(id: &str, proof: &str, at: u64) -> Note {
    let mark = Mark::Done {
        proof: proof.into(),
        changed: None,
        by: None,
    };
    mark.record(&account(), id, at).unwrap()
}

fn ids(steps: Vec<Step>) -> Vec<String> {
    steps.into_iter().map(|s| s.id().to_string()).collect()
}

/// The first done record with a proof closes its step, and one with no content does not
/// (`spec/03-steps.md`, "Closing").
#[test]
fn the_first_done_with_a_proof_closes_its_step() {
    let a = step("a", 10);
    let b = step("b", 11);
    let empty = done(&b.id, " ", 15);
    let first = done(&a.id, "first", 20);
    let later = done(&a.id, "later", 30);
    let records = vec![a.clone(), b.clone(), later, empty, first.clone()];
    let steps = Steps::new(&records, 0);
    let done = steps.get(&a.id).and_then(|s| s.done());
    assert_eq!(done.map(|n| n.id.clone()), Some(first.id));
    let open = steps.all().into_iter().filter(|s| s.closed().is_none());
    assert_eq!(ids(open.collect()), vec![b.id]);
}

/// A prefix of 8 characters that two steps' ids share finds neither, with `no-step`, while a full
/// id still finds its step (`spec/02-cli.md`, "Output and errors").
#[test]
fn a_prefix_two_steps_share_finds_neither() {
    let mut a = step("a", 10);
    let mut b = step("b", 11);
    a.id = format!("abcdefgh{}", "1".repeat(56));
    b.id = format!("abcdefgh{}", "2".repeat(56));
    let records = vec![a.clone(), b];
    let steps = Steps::new(&records, 0);
    let refused = steps.find("abcdefgh").unwrap_err();
    assert!(matches!(refused, Refusal::NoStep(id) if id == "abcdefgh"));
    assert_eq!(steps.find(&a.id).unwrap().id(), a.id);
}

/// A step of an ended trail that is not done or dropped is expired, and a closed one keeps its
/// status.
#[test]
fn an_open_step_of_an_ended_trail_is_expired() {
    let trail = step("trail", 10);
    let mut open = step("open", 11);
    let mut closed = step("closed", 12);
    for child in [&mut open, &mut closed] {
        *child = StepRecord {
            under: Some(StepId::new(&trail.id)),
            ..StepRecord::read(child).unwrap()
        }
        .record(&account(), child.created_at)
        .unwrap();
    }
    let proof = done(&closed.id, "proof", 13);
    let records = vec![trail, open.clone(), closed.clone(), proof];
    let steps = Steps::new(&records, 13 + LIFE);
    let state = |steps: &Steps, id: &str| steps.get(id).unwrap().state();
    assert_eq!(state(&steps, &open.id), State::Expired);
    assert_eq!(state(&steps, &closed.id), State::Done);
    assert_eq!(state(&Steps::new(&records, 13), &open.id), State::Open);
}

/// A store has a closed step once a step under a trail is done or expired; a done trail alone does not
/// count.
#[test]
fn a_step_under_a_trail_closes_the_store() {
    let trail = step("trail", 10);
    let part = StepRecord {
        under: Some(StepId::new(&trail.id)),
        ..StepRecord::read(&step("part", 11)).unwrap()
    }
    .record(&account(), 11)
    .unwrap();
    let ended = done(&trail.id, "proof", 12);
    let records = vec![trail, part.clone(), ended];
    assert!(!Steps::new(&records[..1], 12).closed_any());
    assert!(!Steps::new(&records[..2], 12).closed_any());
    let proof = done(&part.id, "proof", 13);
    let closed = [records.clone(), vec![proof]].concat();
    assert!(Steps::new(&closed, 13).closed_any());
    assert!(Steps::new(&records[..2], 11 + LIFE).closed_any());
}

/// A trail whose steps are all closed is done, but a prompt after the last close opens it again, until
/// 30 minutes pass after it.
#[test]
fn a_prompt_after_the_last_close_opens_its_trail_again() {
    let trail = step("trail", 10);
    let part = StepRecord {
        under: Some(StepId::new(&trail.id)),
        ..StepRecord::read(&step("part", 11)).unwrap()
    }
    .record(&account(), 11)
    .unwrap();
    let proof = done(&part.id, "proof", 12);
    let records = vec![trail.clone(), part, proof];
    let state =
        |records: &[Note], now: u64| Steps::new(records, now).get(&trail.id).unwrap().state();
    assert_eq!(state(&records, 13), State::Done);
    let ask = Mark::Prompt {
        text: "more".into(),
    }
    .record(&account(), &trail.id, 14)
    .unwrap();
    let asked = [records, vec![ask]].concat();
    assert_eq!(state(&asked, 15), State::Open);
    assert_eq!(state(&asked, 14 + LIFE), State::Done);
}
