//! One level of a tree.

use super::CLOSED;
use crate::progress::Phase;
use celis_trails::Step;

/// The steps of one level of a tree: the active ones, then the ended ones, each newest first.
pub(super) struct Level<'a> {
    pub active: Vec<Step<'a>>,
    pub ended: Vec<Step<'a>>,
}

impl<'a> Level<'a> {
    /// The level of `notes`, given oldest first.
    pub fn of(notes: Vec<Step<'a>>) -> Level<'a> {
        let (ended, active) = notes.into_iter().rev().partition(|s| Phase::of(s).ended());
        Level { active, ended }
    }

    /// The ids of the lines a watch prints for it under the line `above`, at any depth: its closed
    /// line, and each step with its own.
    pub fn ids(&self, above: &str) -> Vec<String> {
        let closed = (!self.ended.is_empty()).then(|| format!("{CLOSED}{above}"));
        closed
            .into_iter()
            .chain(ids(&self.active))
            .chain(ids(&self.ended))
            .collect()
    }
}

/// The ids of the lines a watch prints for `steps` and under them, at any depth.
pub(super) fn ids(steps: &[Step]) -> Vec<String> {
    let lines = steps.iter().flat_map(|s| {
        let under = Level::of(s.children()).ids(s.id());
        std::iter::once(s.id().to_string()).chain(under)
    });
    lines.collect()
}
