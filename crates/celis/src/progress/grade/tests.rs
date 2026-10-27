use super::*;
use celis_nostr::{Account, Note};
use celis_store::{Anchor, StepId};
use celis_trails::{Done, Mark, StepRecord, Steps};

fn account() -> Account {
    Account::new(&[1u8; 32]).unwrap()
}

/// A step of `text` at `at`, under `under`.
fn step(text: &str, at: u64, under: Option<&Note>) -> Note {
    StepRecord {
        anchor: Anchor {
            project: "p".into(),
            name: "repo".into(),
        },
        under: under.map(|n| StepId::new(&n.id)),
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

/// `mark` on `step` at `at`.
fn mark(mark: Mark, step: &Note, at: u64) -> Note {
    mark.record(&account(), &step.id, at).unwrap()
}

fn done(step: &Note, at: u64) -> Note {
    mark(Mark::Done(Done::new("it works".into(), None)), step, at)
}

/// A started step took the time from its start to its done.
#[test]
fn a_started_step_took_from_its_start() {
    let trail = step("trail", 10, None);
    let work = step("work", 20, Some(&trail));
    let records = vec![
        trail,
        work.clone(),
        mark(Mark::Start, &work, 100),
        done(&work, 160),
    ];
    let steps = Steps::new(&records, 200);
    let grade = Grade::of(steps.get(&work.id).unwrap());
    assert_eq!(grade, Some(Grade { took: 60 }));
}

/// A step done without a start took the time from its record to its done when no step of its trail
/// closed between them.
#[test]
fn a_step_never_started_took_from_its_record() {
    let trail = step("trail", 10, None);
    let before = step("before", 20, Some(&trail));
    let work = step("work", 40, Some(&trail));
    let records = vec![
        trail,
        before.clone(),
        work.clone(),
        done(&before, 30),
        done(&work, 100),
    ];
    let steps = Steps::new(&records, 200);
    let grade = Grade::of(steps.get(&work.id).unwrap());
    assert_eq!(grade, Some(Grade { took: 60 }));
}

/// A step done without a start took the time from the last close of another step of its trail before
/// its done, so that the wait of a step planned ahead does not count.
#[test]
fn a_step_never_started_took_from_the_close_before_it() {
    let trail = step("trail", 10, None);
    let ask = step("ask", 15, Some(&trail));
    let first = step("first", 20, Some(&ask));
    let second = step("second", 20, Some(&trail));
    let work = step("work", 20, Some(&trail));
    let records = vec![
        trail,
        ask,
        first.clone(),
        second.clone(),
        work.clone(),
        done(&first, 5_000),
        done(&second, 9_000),
        done(&work, 9_120),
    ];
    let steps = Steps::new(&records, 10_000);
    let grade = Grade::of(steps.get(&work.id).unwrap());
    assert_eq!(grade, Some(Grade { took: 120 }));
}

/// A step done without a start, with steps under it, took the time from its record to its done.
#[test]
fn a_step_with_steps_under_it_took_from_its_record() {
    let trail = step("trail", 10, None);
    let ask = step("ask", 20, Some(&trail));
    let part = step("part", 20, Some(&ask));
    let records = vec![
        trail,
        ask.clone(),
        part.clone(),
        done(&part, 80),
        done(&ask, 80),
    ];
    let steps = Steps::new(&records, 200);
    let grade = Grade::of(steps.get(&ask.id).unwrap());
    assert_eq!(grade, Some(Grade { took: 60 }));
}

/// Of steps done in the same second without a start, the first by its done record took the time from
/// its record, and each next one none.
#[test]
fn steps_done_in_one_second_count_their_wait_once() {
    let trail = step("trail", 10, None);
    let one = step("one", 20, Some(&trail));
    let two = step("two", 20, Some(&trail));
    let (a, b) = (done(&one, 80), done(&two, 80));
    let records = vec![trail, one.clone(), two.clone(), a.clone(), b.clone()];
    let steps = Steps::new(&records, 200);
    let took = |s: &Note| Grade::of(steps.get(&s.id).unwrap()).unwrap().took;
    let (first, next) = match a.id < b.id {
        true => (&one, &two),
        false => (&two, &one),
    };
    assert_eq!((took(first), took(next)), (60, 0));
}
