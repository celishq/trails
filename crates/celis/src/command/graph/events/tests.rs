use super::*;
use celis_nostr::{Account, Note};
use celis_trails::{Anchor, Mark, Rule, Scope, StepId, StepRecord, Steps};

fn account() -> Account {
    Account::new(&[1u8; 32]).unwrap()
}

fn step(under: Option<&Note>, at: u64, content: &str) -> Note {
    StepRecord {
        anchor: Anchor {
            project: "p".into(),
            name: "repo".into(),
        },
        under: under.map(|n| StepId::new(&n.id)),
        replaces: None,
        uses: Vec::new(),
        places: Vec::new(),
        check: None,
        session: None,
        content: content.into(),
    }
    .record(&account(), at)
    .unwrap()
}

fn mark(mark: Mark, step: &Note, at: u64) -> Note {
    mark.record(&account(), &step.id, at).unwrap()
}

/// A events holds each step added, started and closed and each rule of its trail, by the second, then
/// added before started before closed, then by the order of the steps; it tells where a step stands.
#[test]
fn a_history_holds_the_events_of_a_trail_in_the_order_they_happened() {
    let now = 1_800_000_000;
    let trail = step(None, now - 100, "trail");
    let page = step(Some(&trail), now - 90, "page");
    let logo = step(Some(&trail), now - 85, "logo");
    let started = mark(Mark::Start, &page, now - 90);
    let done = mark(
        Mark::Done {
            proof: "it shows".into(),
            changed: None,
        },
        &page,
        now - 60,
    );
    let dropped = mark(
        Mark::Drop {
            reason: "no need".into(),
        },
        &logo,
        now - 50,
    );
    let rule = Rule {
        anchor: Anchor {
            project: "p".into(),
            name: "repo".into(),
        },
        scope: Scope::Trail(trail.id.clone()),
        replaces: None,
        content: "When it shows: keep it, because it works.".into(),
        paths: vec![],
        situation: false,
        promoted: false,
    }
    .record(&account(), now - 70)
    .unwrap();
    let records = [trail.clone(), page, logo, started, done, dropped, rule];
    let steps = Steps::new(&records, now);
    let rules = Rules::new(&records);
    let events = Events::of(steps.get(&trail.id).unwrap(), &rules);
    let happened: Vec<(Kind, &str)> = events
        .all
        .iter()
        .map(|e| (e.kind, e.text.as_str()))
        .collect();
    assert_eq!(
        happened,
        [
            (Kind::Added, "page"),
            (Kind::Started, "page"),
            (Kind::Added, "logo"),
            (Kind::Rule, "When it shows: keep it"),
            (Kind::Done, "page"),
            (Kind::Dropped, "logo"),
        ]
    );
    let page = &events.all[0].id;
    assert_eq!(events.state(page, 0), None);
    assert_eq!(events.state(page, 1), Some(State::Open));
    assert_eq!(events.state(page, 2), Some(State::Working));
    assert_eq!(events.state(page, 6), Some(State::Done));
}
