use super::super::{Tree, View};
use super::Layout;
use crate::progress::{Model, Reuse};
use crate::text::clock::day;
use celis_nostr::{Account, Note};
use celis_patterns::{PatternRecord, Patterns, Scope};
use celis_store::{Anchor, StepId};
use celis_trails::{Mark, Notes, StepRecord, Steps};
use std::cell::Cell;

/// The time the newest record was written.
const NOW: u64 = 1_800_000_000;

fn account() -> Account {
    Account::new(&[1u8; 32]).unwrap()
}

fn step(under: Option<&Note>, uses: &[&Note], at: u64) -> Note {
    StepRecord {
        anchor: Anchor {
            project: "p".into(),
            name: "repo".into(),
        },
        under: under.map(|n| StepId::new(&n.id)),
        replaces: None,
        uses: uses.iter().map(|n| n.id.clone()).collect(),
        misled: Vec::new(),
        places: Vec::new(),
        check: None,
        session: None,
        agent: None,
        content: format!("step at {at}"),
    }
    .record(&account(), at)
    .unwrap()
}

/// Prints every trail of `records` by day into `layout`, as `--all` shows them.
fn print(records: &[Note], layout: &Layout) {
    let steps = Steps::new(records, NOW);
    let (patterns, notes) = (Patterns::new(records), Notes::new(records));
    let view = View {
        all: true,
        ..View::default()
    };
    let tree = Tree {
        steps: &steps,
        patterns: &patterns,
        notes: &notes,
        layout: Some(layout),
        grouped: true,
        width: Some(80),
        view: &view,
        printed: Default::default(),
        picked: Default::default(),
        spun: Cell::default(),
        order: Default::default(),
        next: Cell::default(),
    };
    let roots: Vec<_> = steps.roots().into_iter().rev().collect();
    tree.print(&roots, "trail", false, &mut Vec::new()).unwrap();
}

/// The days whose groups `layout` keeps after `landed` joins `records`.
fn kept(records: &[Note], landed: &[Note]) -> Vec<String> {
    let layout = Layout::default();
    print(records, &layout);
    let all: Vec<Note> = records.iter().chain(landed).cloned().collect();
    let (steps, patterns) = (Steps::new(&all, NOW), Patterns::new(&all));
    let notes = Notes::new(&all);
    let reuse = Reuse::new(&steps);
    let model = Model {
        steps: &steps,
        patterns: &patterns,
        notes: &notes,
        reuse: &reuse,
    };
    layout.renew(&model, Some(landed));
    let mut days: Vec<String> = layout.groups.borrow().keys().cloned().collect();
    days.sort();
    days
}

/// A step or a mark in one trail forgets only the day group of that trail; a pattern, a step that uses
/// another, or records of two trails forget every group.
#[test]
fn a_record_forgets_only_the_group_of_its_trail() {
    let (old, new) = (NOW - 3 * 86_400, NOW - 60);
    let first = step(None, &[], old);
    let part = step(Some(&first), &[], old);
    let second = step(None, &[], new);
    let records = [first.clone(), part.clone(), second.clone()];
    let both = vec![day(old), day(new)];
    let mut only_old = both.clone();
    only_old.retain(|d| *d == day(old));

    assert_eq!(kept(&records, &[step(Some(&second), &[], NOW)]), only_old);
    let start = Mark::Start.record(&account(), &second.id, NOW).unwrap();
    assert_eq!(kept(&records, &[start]), only_old);
    assert_eq!(kept(&records, &[]), both);

    let pattern = PatternRecord {
        anchor: Anchor {
            project: "p".into(),
            name: "repo".into(),
        },
        scope: Scope::Project,
        replaces: None,
        content: "a rule".into(),
        paths: vec![],
        within: vec![],
        examples: Default::default(),
    }
    .record(&account(), NOW)
    .unwrap();
    assert!(kept(&records, &[pattern]).is_empty());
    assert!(kept(&records, &[step(Some(&second), &[&part], NOW)]).is_empty());
    let two = [step(Some(&second), &[], NOW), step(Some(&first), &[], NOW)];
    assert!(kept(&records, &two).is_empty());
}
