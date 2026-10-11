use super::*;
use crate::Anchor;
use celis_wire::Account;

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
        fixes: None,
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
    let steps = Steps {
        records: &records,
        now: 0,
    };
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
    let steps = Steps {
        records: &records,
        now: 0,
    };
    let refused = steps.find("abcdefgh").unwrap_err();
    assert_eq!((refused.name.as_str(), refused.code), ("no-step", 1));
    assert_eq!(steps.find(&a.id).unwrap().id, a.id);
}
