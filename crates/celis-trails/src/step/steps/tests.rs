use super::*;
use crate::State;
use celis_nostr::Account;
use celis_store::{Anchor, StepId};

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
        misled: Vec::new(),
        places: Vec::new(),
        check: None,
        session: None,
        agent: None,
        content: text.into(),
    }
    .record(&account(), at)
    .unwrap()
}

fn done(id: &str, proof: &str, at: u64) -> Note {
    let mark = Mark::Done(crate::Done::new(proof.into(), None));
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
    let done = steps.get(&a.id).and_then(|s| s.done_record());
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

/// A step of an ended trail that is not done or dropped stays open and is stale, and a closed one keeps
/// its state.
#[test]
fn an_open_step_of_an_ended_trail_is_stale() {
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
    assert_eq!(state(&steps, &open.id), State::Open);
    assert!(steps.get(&open.id).unwrap().stale());
    assert_eq!(state(&steps, &closed.id), State::Done);
    assert!(!steps.get(&closed.id).unwrap().stale());
    assert!(!Steps::new(&records, 13).get(&open.id).unwrap().stale());
}

/// An open step is active until its trail ends, and a closed one is not.
#[test]
fn an_open_step_is_active_until_its_trail_ends() {
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
    let active = |now: u64, id: &str| Steps::new(&records, now).get(id).unwrap().active();
    assert!(active(13, &open.id));
    assert!(!active(13 + LIFE, &open.id));
    assert!(!active(13, &closed.id));
}

/// A started step stays open, and works until its trail ends (`spec/03-steps.md`, "Starting").
#[test]
fn a_started_step_stays_open() {
    let trail = step("trail", 10);
    let part = StepRecord {
        under: Some(StepId::new(&trail.id)),
        ..StepRecord::read(&step("part", 11)).unwrap()
    }
    .record(&account(), 11)
    .unwrap();
    let start = Mark::Start.record(&account(), &part.id, 12).unwrap();
    let records = vec![trail.clone(), part.clone(), start];
    let steps = Steps::new(&records, 13);
    let started = steps.get(&part.id).unwrap();
    assert_eq!(started.state(), State::Open);
    assert!(started.works() && steps.get(&trail.id).unwrap().works());
    assert!(!Steps::new(&records, 12 + LIFE)
        .get(&part.id)
        .unwrap()
        .works());
}

/// A done trail that a drop names is dropped, and keeps its proof (`spec/03-steps.md`, "Dropping").
#[test]
fn a_done_trail_that_a_drop_names_is_dropped() {
    let trail = step("trail", 10);
    let proof = done(&trail.id, "proof", 11);
    let drop = Mark::Drop {
        reason: "stale".into(),
    }
    .record(&account(), &trail.id, 12)
    .unwrap();
    let records = vec![trail.clone(), proof.clone(), drop];
    let steps = Steps::new(&records, 13);
    let dropped = steps.get(&trail.id).unwrap();
    assert_eq!(dropped.state(), State::Dropped);
    assert_eq!(dropped.done_record().map(|n| &n.id), Some(&proof.id));
}

/// A store has a closed step once a step under a trail is done or stale; a done trail alone does not
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

/// A step under a done trail reopens it, while it is open or newer than the trail's newest done; a
/// later done settles it again, and a step of the same second as the done does not reopen it
/// (`spec/03-steps.md`, "Closing").
#[test]
fn a_step_under_a_done_trail_reopens_it() {
    let under = |text: &str, trail: &Note, at: u64| {
        StepRecord {
            under: Some(StepId::new(&trail.id)),
            ..StepRecord::read(&step(text, at)).unwrap()
        }
        .record(&account(), at)
        .unwrap()
    };
    let trail = step("trail", 10);
    let first = done(&trail.id, "first", 20);
    let same = under("same second", &trail, 20);
    let same_done = done(&same.id, "proof", 20);
    let settled = vec![trail.clone(), first.clone(), same, same_done.clone()];
    let state = |records: &[Note]| Steps::new(records, 0).get(&trail.id).unwrap().state();
    assert_eq!(state(&settled), State::Done);
    let follow = under("follow-up", &trail, 30);
    let opened = [settled.clone(), vec![follow.clone()]].concat();
    assert_eq!(state(&opened), State::Open);
    let follow_done = done(&follow.id, "proof", 40);
    let closed = [opened.clone(), vec![follow_done]].concat();
    assert_eq!(state(&closed), State::Open);
    let again = done(&trail.id, "again", 50);
    let records = [closed, vec![again.clone()]].concat();
    let steps = Steps::new(&records, 0);
    let settled = steps.get(&trail.id).unwrap();
    assert_eq!(settled.state(), State::Done);
    assert_eq!(settled.done_record().map(|n| &n.id), Some(&again.id));
}
