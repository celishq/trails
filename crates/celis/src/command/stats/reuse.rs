//! The one measure of pattern reuse that every view of stats reads (`DESIGN.md`, decision 19).

use super::outcomes::{percent, share};
use celis_trails::{State, Step, Steps};
use std::collections::BTreeMap;

/// The pattern reuse of the asks done in a span: the asks done, those whose tree names a pattern, and
/// how often the steps of those trees name each pattern.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Reuse {
    pub asks: usize,
    pub reused: usize,
    pub uses: BTreeMap<String, usize>,
}

impl Reuse {
    /// The reuse of the asks of `steps` done from `from` up to `to`.
    pub fn between(steps: &Steps, from: u64, to: u64) -> Reuse {
        let asks: Vec<Step> = steps
            .roots()
            .into_iter()
            .filter(|s| s.state() == State::Done)
            .filter(|s| {
                s.done_record()
                    .is_some_and(|d| (from..to).contains(&d.created_at))
            })
            .collect();
        Reuse::of(&asks)
    }

    /// The reuse of the done asks `asks`.
    pub fn of(asks: &[Step]) -> Reuse {
        let mut reuse = Reuse {
            asks: asks.len(),
            ..Reuse::default()
        };
        for ask in asks {
            let named: Vec<&str> = ask.tree().into_iter().flat_map(|s| s.uses()).collect();
            reuse.reused += usize::from(!named.is_empty());
            for id in named {
                *reuse.uses.entry(id.to_string()).or_default() += 1;
            }
        }
        reuse
    }

    /// The reuse of every span of `spans` together.
    pub fn sum(spans: &[Reuse]) -> Reuse {
        let mut sum = Reuse::default();
        for span in spans {
            sum.asks += span.asks;
            sum.reused += span.reused;
            for (id, n) in &span.uses {
                *sum.uses.entry(id.clone()).or_default() += n;
            }
        }
        sum
    }

    /// The pattern uses.
    pub fn count(&self) -> usize {
        self.uses.values().sum()
    }

    /// The reused asks among the done ones, in whole per cent; none with no ask done.
    pub fn percent(&self) -> Option<i64> {
        (self.asks > 0).then(|| percent(self.reused, self.asks))
    }

    /// The share as a view shows it (`outcomes::share`).
    pub fn text(&self) -> String {
        share(self.reused, self.asks)
    }

    /// Each pattern with its uses, most first, then by id.
    pub fn top(&self) -> Vec<(&str, usize)> {
        let mut top: Vec<(&str, usize)> =
            self.uses.iter().map(|(id, n)| (id.as_str(), *n)).collect();
        top.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        top
    }
}

#[cfg(test)]
mod tests;
