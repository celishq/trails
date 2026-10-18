//! What happened on one trail, in the order it happened.

use celis_trails::{Decisions, State, Step, Trail};

/// The events of one trail in the order they happened: each step added, started and closed, and each
/// rule written for it.
pub(crate) struct Events {
    pub all: Vec<Event>,
}

/// One thing that happened on a trail: what, to the step or rule `id`, with its title or rule.
pub(crate) struct Event {
    pub kind: Kind,
    pub id: String,
    pub text: String,
}

/// What happened: a step added, a rule written, a step started, done or dropped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Kind {
    Added,
    Rule,
    Started,
    Done,
    Dropped,
}

impl Events {
    /// The events of the trail `head` and of the rules `decisions` holds for it, by the second each
    /// happened, then by kind, then by the order of its steps.
    pub fn of(head: Step, decisions: &Decisions) -> Events {
        let first = Trail::new(head).first().map(|s| s.id());
        let mut timed = Vec::new();
        let steps = head.tree().into_iter().skip(1);
        for (n, step) in steps.filter(|s| Some(s.id()) != first).enumerate() {
            let event = |kind| Event {
                kind,
                id: step.id().to_string(),
                text: step.title().to_string(),
            };
            timed.push((step.note().created_at, n, event(Kind::Added)));
            if let Some(start) = step.start() {
                timed.push((start.created_at, n, event(Kind::Started)));
            }
            if let Some(done) = step.done() {
                timed.push((done.created_at, n, event(Kind::Done)));
            }
            if let Some(dropped) = step.dropped() {
                timed.push((dropped.created_at, n, event(Kind::Dropped)));
            }
        }
        for (note, decision) in decisions.rules(head) {
            let event = Event {
                kind: Kind::Rule,
                id: note.id.clone(),
                text: decision.brief().to_string(),
            };
            timed.push((note.created_at, usize::MAX, event));
        }
        timed.sort_by_key(|(at, n, event)| (*at, event.kind, *n));
        Events {
            all: timed.into_iter().map(|(_, _, event)| event).collect(),
        }
    }

    /// Where the step `id` stands after the first `shown` events; `None` before it was added.
    pub fn state(&self, id: &str, shown: usize) -> Option<State> {
        let events = self.all.iter().take(shown).filter(|e| e.id == id);
        events.fold(None, |_, event| match event.kind {
            Kind::Added | Kind::Rule => Some(State::Open),
            Kind::Started => Some(State::Working),
            Kind::Done => Some(State::Done),
            Kind::Dropped => Some(State::Dropped),
        })
    }
}

#[cfg(test)]
mod tests;
