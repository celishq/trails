//! What the graph view saw change since it opened, which its animations play from.

use crate::progress::Phase;
use celis_trails::Step;
use std::collections::HashMap;
use std::time::Instant;

/// What the view saw since it opened: when it opened; for each step the state it last saw, the state
/// before, when it changed and when it was added; when each pattern landed; when each trail last moved and
/// was last shown. A moment before the view opened is `None`, so what stood then plays no animation.
pub(super) struct Seen {
    pub opened: Instant,
    steps: HashMap<String, Change>,
    patterns: HashMap<String, Option<Instant>>,
    moved: HashMap<String, Instant>,
    shown: HashMap<String, Instant>,
    first: bool,
}

/// One step as the view saw it: its state, the state before, and when it changed and was added.
#[derive(Clone, Copy)]
pub(super) struct Change {
    pub state: Phase,
    pub before: Option<Phase>,
    pub at: Option<Instant>,
    pub added: Option<Instant>,
}

impl Seen {
    pub fn new() -> Seen {
        Seen {
            opened: Instant::now(),
            steps: HashMap::new(),
            patterns: HashMap::new(),
            moved: HashMap::new(),
            shown: HashMap::new(),
            first: true,
        }
    }

    /// A view that opened a minute ago, so what it notes first has long arrived.
    pub fn settled() -> Seen {
        let mut seen = Seen::new();
        seen.opened = Instant::now() - std::time::Duration::from_secs(60);
        seen
    }

    /// Whether something still moves at `now`: the view opened, a step changed or a pattern landed within
    /// the last two seconds.
    pub fn lively(&self, now: Instant) -> bool {
        let recent = |at: &Instant| now.duration_since(*at).as_secs_f64() < 2.0;
        recent(&self.opened)
            || self
                .steps
                .values()
                .any(|c| c.at.as_ref().is_some_and(recent))
            || self.patterns.values().flatten().any(recent)
    }

    /// Notes each step of `trails` and each pattern `patterns` gives a trail at `now`: a step it had not
    /// seen, or one whose state changed, and a pattern it had not seen, change now; on the first look,
    /// all stood before.
    pub fn note<'a>(
        &mut self,
        trails: &[Step<'a>],
        patterns: impl Fn(Step<'a>) -> Vec<String>,
        now: Instant,
    ) {
        let at = (!self.first).then_some(now);
        for trail in trails {
            let mut moved = false;
            for step in trail.tree() {
                let state = Phase::of(&step);
                match self.steps.get_mut(step.id()) {
                    Some(seen) if seen.state != state => {
                        *seen = Change {
                            state,
                            before: Some(seen.state),
                            at,
                            added: seen.added,
                        };
                        moved = true;
                    }
                    Some(_) => {}
                    None => {
                        let change = Change {
                            state,
                            before: None,
                            at,
                            added: at,
                        };
                        self.steps.insert(step.id().to_string(), change);
                        moved = true;
                    }
                }
            }
            for pattern in patterns(*trail) {
                if let std::collections::hash_map::Entry::Vacant(entry) =
                    self.patterns.entry(pattern)
                {
                    entry.insert(at);
                    moved = true;
                }
            }
            if let (true, Some(at)) = (moved, at) {
                self.moved.insert(trail.id().to_string(), at);
            }
        }
        self.first = false;
    }

    /// How the view saw the step `id`.
    pub fn step(&self, id: &str) -> Option<Change> {
        self.steps.get(id).copied()
    }

    /// The seconds since the step `id` changed at `now`; `None` when it stood so before the view opened.
    pub fn changed(&self, id: &str, now: Instant) -> Option<f64> {
        let at = self.steps.get(id)?.at?;
        Some(now.duration_since(at).as_secs_f64())
    }

    /// The seconds since the step `id` was added at `now`; `None` when it stood before the view opened.
    pub fn added(&self, id: &str, now: Instant) -> Option<f64> {
        let at = self.steps.get(id)?.added?;
        Some(now.duration_since(at).as_secs_f64())
    }

    /// The seconds since the pattern `id` landed at `now`; `None` when it stood before the view opened.
    pub fn landed(&self, id: &str, now: Instant) -> Option<f64> {
        let at = (*self.patterns.get(id)?)?;
        Some(now.duration_since(at).as_secs_f64())
    }

    /// The seconds since the trail `id` last moved at `now`; `None` when it has not since the view
    /// opened.
    pub fn moved(&self, id: &str, now: Instant) -> Option<f64> {
        let at = self.moved.get(id)?;
        Some(now.duration_since(*at).as_secs_f64())
    }

    /// Notes that the trail `id` is shown at `now`.
    pub fn show(&mut self, id: &str, now: Instant) {
        self.shown.insert(id.to_string(), now);
    }

    /// Whether the trail `id` moved since it was last shown.
    pub fn news(&self, id: &str) -> bool {
        let moved = self.moved.get(id);
        moved.is_some_and(|m| self.shown.get(id).is_none_or(|s| m > s))
    }
}

#[cfg(test)]
mod tests;
