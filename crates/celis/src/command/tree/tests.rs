use super::*;
use celis_nostr::Account;
use celis_trails::{Anchor, Mark};

fn account() -> Account {
    Account::new(&[1u8; 32]).unwrap()
}

fn step(under: Option<&Note>, at: u64) -> Note {
    Step {
        anchor: Anchor {
            project: "p".into(),
            name: "repo".into(),
        },
        under: under.map(|n| n.id.clone()),
        replaces: None,
        places: Vec::new(),
        check: None,
        session: None,
        content: "a step".into(),
    }
    .record(&account(), at)
    .unwrap()
}

/// The time the tree of `records` at `now` changes next.
fn next(records: &[Note], now: u64) -> Option<u64> {
    let steps = Steps::new(records, now);
    let decisions = Decisions::new(records);
    let tree = Tree {
        steps: &steps,
        decisions: &decisions,
        colour: false,
        full: false,
        fold: false,
        all: false,
        named: false,
        width: None,
        next: Cell::default(),
    };
    tree.print(&[&records[0]], "trail", false, &mut Vec::new())
        .unwrap();
    tree.next.get()
}

/// The tree changes next when the first time it shows changes: a time in seconds the next second, a time
/// in hours the next minute, and a tree with no time that runs, none (`spec/trails/04-trails.md`,
/// "Watching").
#[test]
fn the_tree_changes_next_when_its_first_time_changes() {
    let now = 1_800_000_000;
    let young = step(None, now - 90);
    assert_eq!(next(&[young], now), Some(now + 1));

    let old = step(None, now - 7_200);
    let recent = step(Some(&old), now - 60);
    let midnight = midnight(now) + 86_400;
    assert_eq!(next(&[old, recent], now), Some((now + 60).min(midnight)));

    let past = step(None, now - 3 * 86_400);
    let proof = Mark::Done {
        proof: "proof".into(),
        changed: None,
    }
    .record(&account(), &past.id, now - 3 * 86_400)
    .unwrap();
    assert_eq!(next(&[past, proof], now), None);
}
