//! The lifecycle of the patterns: what happened to them by day, week and month.

use crate::progress::Model;
use crate::text::clock::{midnight, month};
use celis_patterns::{Pattern, PatternEnd};
use std::collections::HashSet;

/// The seconds of a day.
const DAY: u64 = 86_400;

/// One span of the lifecycle: `day`, `week` or `month`, its first second, and the patterns written,
/// replaced, dropped and promoted in it, with the recalls, uses and misled steps of them.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Lifecycle {
    pub span: &'static str,
    pub from: u64,
    pub written: usize,
    pub replaced: usize,
    pub dropped: usize,
    pub promoted: usize,
    pub recalled: usize,
    pub used: usize,
    pub misled: usize,
}

/// What happened to a pattern at a second.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Event {
    Written,
    Replaced,
    Dropped,
    Promoted,
    Recalled,
    Used,
    Misled,
}

impl Lifecycle {
    /// The last 7 days, 4 weeks and 6 months of the patterns in scope in `project`, or of every
    /// pattern with none, each kind oldest first (CIP-02, "Stats").
    pub fn of(model: &Model, project: Option<&str>) -> Vec<Lifecycle> {
        let events = events(model, project);
        let today = midnight(model.steps.now());
        let mut spans = Vec::new();
        for k in (0..7).rev() {
            let from = today - k * DAY;
            spans.push(("day", from, midnight(from + DAY + 12 * 3_600)));
        }
        for k in (0..4).rev() {
            let to = midnight(today + DAY + 12 * 3_600) - k * 7 * DAY;
            spans.push(("week", to - 7 * DAY, to));
        }
        let mut starts = vec![month(today)];
        for _ in 0..6 {
            let last = *starts.last().unwrap_or(&today);
            starts.push(month(last.saturating_sub(DAY)));
        }
        starts.reverse();
        let next = month(today + 32 * DAY);
        let ends = starts[2..].iter().copied().chain([next]);
        for (from, to) in starts[1..].iter().copied().zip(ends) {
            spans.push(("month", from, to));
        }
        spans
            .into_iter()
            .map(|(span, from, to)| {
                let mut life = Lifecycle {
                    span,
                    from,
                    ..Lifecycle::default()
                };
                let inside = events.iter().filter(|(at, _)| (from..to).contains(at));
                for (_, event) in inside {
                    life.count(*event);
                }
                life
            })
            .collect()
    }

    /// Takes in one `event`.
    fn count(&mut self, event: Event) {
        match event {
            Event::Written => self.written += 1,
            Event::Replaced => self.replaced += 1,
            Event::Dropped => self.dropped += 1,
            Event::Promoted => {
                self.dropped += 1;
                self.promoted += 1;
            }
            Event::Recalled => self.recalled += 1,
            Event::Used => self.used += 1,
            Event::Misled => self.misled += 1,
        }
    }
}

/// Whether the drop of `pattern` promoted it into a file of the project.
pub(crate) fn promoted(pattern: &Pattern) -> bool {
    matches!(pattern.end, Some(PatternEnd::Dropped(drop)) if drop.content.starts_with("promoted to"))
}

/// Each event of the patterns in scope in `project`, or of every pattern, with its second.
fn events(model: &Model, project: Option<&str>) -> Vec<(u64, Event)> {
    let patterns = match project {
        Some(project) => model.patterns.view(project).scoped(),
        None => model.patterns.every().to_vec(),
    };
    let ids: HashSet<&str> = patterns.iter().map(|p| p.id()).collect();
    let mut events = Vec::new();
    for pattern in &patterns {
        let made = match pattern.record.replaces {
            Some(_) => Event::Replaced,
            None => Event::Written,
        };
        events.push((pattern.note.created_at, made));
        if let Some(PatternEnd::Dropped(drop)) = pattern.end {
            let ended = match promoted(pattern) {
                true => Event::Promoted,
                false => Event::Dropped,
            };
            events.push((drop.created_at, ended));
        }
    }
    for step in model.steps.all() {
        let at = step.note().created_at;
        let uses = step.uses().into_iter().filter(|p| ids.contains(p));
        events.extend(uses.map(|_| (at, Event::Used)));
        let recalls = step.recalls().into_iter().filter(|(p, _)| ids.contains(p));
        events.extend(recalls.map(|(_, at)| (at, Event::Recalled)));
        let misled = step.misled().into_iter().filter(|(p, _)| ids.contains(p));
        events.extend(misled.map(|(_, at)| (at, Event::Misled)));
    }
    events
}

#[cfg(test)]
mod tests;
