//! One level of a tree.

use crate::progress::Phase;
use celis_trails::Step;

/// The steps of one level of a tree: the active ones, then the ended ones, each newest first, and every
/// one in the order they came.
pub(super) struct Level<'a> {
    pub active: Vec<Step<'a>>,
    pub ended: Vec<Step<'a>>,
    pub order: Vec<Step<'a>>,
}

impl<'a> Level<'a> {
    /// The level of `notes`, given oldest first.
    pub fn of(notes: Vec<Step<'a>>) -> Level<'a> {
        let order = notes.clone();
        let (ended, active) = notes.into_iter().rev().partition(|s| Phase::of(s).ended());
        Level {
            active,
            ended,
            order,
        }
    }

    /// The ids of the lines a watch prints for it, at any depth: each step with its own.
    pub fn ids(&self) -> Vec<String> {
        ids(&self.active)
            .into_iter()
            .chain(ids(&self.ended))
            .collect()
    }
}

/// The ids of the lines a watch prints for `steps` and under them, at any depth.
pub(super) fn ids(steps: &[Step]) -> Vec<String> {
    let lines = steps.iter().flat_map(|s| {
        let under = Level::of(s.children()).ids();
        std::iter::once(s.id().to_string()).chain(under)
    });
    lines.collect()
}
