//! The route of a step on a board: the step as its card.

use super::{route_head, Board, NARROW};
use crate::terminal::live::Page;
use celis_store::records::id::short;
use celis_trails::Step;
use face::face;

impl Board<'_> {
    /// The page of the route of `step`: its head, then its card.
    pub(super) fn route(&self, step: Step, width: usize) -> Page {
        let name = &step.record().anchor.name;
        let mut lines = route_head(&short(step.id()), name, width);
        lines.extend(face(step, width, width < NARROW));
        Page {
            lines,
            keys: "q quit".into(),
            ..Page::default()
        }
    }
}

mod face;
