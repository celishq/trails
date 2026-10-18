//! CIP-06, "Uses": what one `uses` of a step names.

use crate::Step;

/// What one `uses` of a step names: a step it follows, else the id of a decision it uses.
#[derive(Clone, Copy)]
pub enum Used<'a> {
    Follow(Step<'a>),
    Decision(&'a str),
}
