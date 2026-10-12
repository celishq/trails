use super::*;
use crate::Anchor;
use celis_nostr::Account;

fn account() -> Account {
    Account::new(&[1u8; 32]).unwrap()
}

fn step(text: &str, at: u64) -> Note {
    Step {
        anchor: Anchor {
            project: "p".into(),
            name: "repo".into(),
        },
        under: None,
        replaces: None,
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
    };
    mark.record(&account(), id, at).unwrap()
}

fn ids(notes: Vec<&Note>) -> Vec<String> {
    notes.into_iter().map(|n| n.id.clone()).collect()
}

/// The first done record with a proof closes its step, and one with no content does not
/// (`spec/trails/12-steps.md`, "Closing").
#[test]
fn the_first_done_with_a_proof_closes_its_step() {
    let a = step("a", 10);
    let b = step("b", 11);
    let empty = done(&b.id, " ", 15);
    let first = done(&a.id, "first", 20);
    let later = done(&a.id, "later", 30);
    let records = vec![a.clone(), b.clone(), later, empty, first.clone()];
    let steps = Steps::new(&records, 0);
    assert_eq!(steps.done(&a.id).map(|n| n.id.clone()), Some(first.id));
    assert_eq!(ids(steps.open()), vec![b.id]);
}

/// A prefix of 8 characters that two steps' ids share finds neither, with `no-step`, while a full
/// id still finds its step (`spec/trails/12-steps.md`, "The commands").
#[test]
fn a_prefix_two_steps_share_finds_neither() {
    let mut a = step("a", 10);
    let mut b = step("b", 11);
    a.id = format!("abcdefgh{}", "1".repeat(56));
    b.id = format!("abcdefgh{}", "2".repeat(56));
    let records = vec![a.clone(), b];
    let steps = Steps::new(&records, 0);
    let refused = steps.find("abcdefgh").unwrap_err();
    assert_eq!((refused.name.as_str(), refused.code), ("no-step", 1));
    assert_eq!(steps.find(&a.id).unwrap().id, a.id);
}

/// A step of an ended trail that is not done or dropped is expired, and a closed one keeps its status
/// (`spec/trails/14-trails.md`, "The commands").
#[test]
fn an_open_step_of_an_ended_trail_is_expired() {
    let trail = step("trail", 10);
    let mut open = step("open", 11);
    let mut closed = step("closed", 12);
    for child in [&mut open, &mut closed] {
        *child = Step {
            under: Some(trail.id.clone()),
            ..Step::read(child).unwrap()
        }
        .record(&account(), child.created_at)
        .unwrap();
    }
    let proof = done(&closed.id, "proof", 13);
    let records = vec![trail, open.clone(), closed.clone(), proof];
    let steps = Steps::new(&records, 13 + LIFE);
    assert_eq!(steps.status(&open.id), Status::Expired);
    assert_eq!(steps.status(&closed.id), Status::Done);
    assert_eq!(Steps::new(&records, 13).status(&open.id), Status::Open);
}
