use super::*;
use celis_nostr::{Account, Note};
use celis_store::Anchor;
use celis_trails::{Mark, Notes, StepRecord};

fn account() -> Account {
    Account::new(&[1u8; 32]).unwrap()
}

fn step(under: Option<&Note>, at: u64, content: &str) -> Note {
    StepRecord {
        anchor: Anchor {
            project: "p".into(),
            name: "repo".into(),
        },
        under: under.map(|n| celis_store::StepId::new(&n.id)),
        replaces: None,
        uses: Vec::new(),
        misled: Vec::new(),
        places: Vec::new(),
        check: None,
        session: None,
        content: content.into(),
    }
    .record(&account(), at)
    .unwrap()
}

/// The time the tree of `records` at `now` changes next.
fn next(records: &[Note], now: u64) -> Option<u64> {
    let steps = Steps::new(records, now);
    let (patterns, notes) = (Patterns::new(records), Notes::new(records));
    let tree = Tree {
        steps: &steps,
        patterns: &patterns,
        notes: &notes,
        layout: None,
        grouped: false,
        width: None,
        view: &View::default(),
        printed: Default::default(),
        picked: Default::default(),
        spun: Cell::default(),
        order: Default::default(),
        next: Cell::default(),
    };
    tree.print(
        &[steps.get(&records[0].id).unwrap()],
        "trail",
        false,
        &mut Vec::new(),
    )
    .unwrap();
    tree.next.get()
}

/// The tree changes next when the first time it shows changes: a started step's time in seconds the next
/// second, in hours the next minute, else when the trail ends; a closed tree, never.
#[test]
fn the_tree_changes_next_when_its_first_time_changes() {
    let now = 1_800_000_000;
    let start = |step: &Note, at: u64| Mark::Start.record(&account(), &step.id, at).unwrap();
    let young = step(None, now - 90, "trail");
    let part = step(Some(&young), now - 90, "part");
    let started = start(&part, now - 30);
    assert_eq!(next(&[young, part, started], now), Some(now + 1));

    let old = step(None, now - 7_200, "trail");
    let long = step(Some(&old), now - 7_200, "long");
    let started = start(&long, now - 7_200);
    let recent = step(Some(&old), now - 60, "recent");
    assert_eq!(next(&[old, long, started, recent], now), Some(now + 60));

    let idle = step(None, now - 600, "trail");
    let open = step(Some(&idle), now - 600, "open");
    assert_eq!(next(&[idle, open], now), Some(now - 600 + LIFE));

    let past = step(None, now - 3 * 86_400, "trail");
    let proof = Mark::Done(celis_trails::Done::new("proof".into(), None))
        .record(&account(), &past.id, now - 3 * 86_400)
        .unwrap();
    assert_eq!(next(&[past, proof], now), None);
}
