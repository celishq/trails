//! CIP-06, "Uses": what one `uses` of a step names.

use crate::Step;

/// What one `uses` of a step names: a step it follows, or the id of a pattern it uses.
#[derive(Clone, Copy)]
pub enum Used<'a> {
    Follow(Step<'a>),
    Pattern(&'a str),
}
