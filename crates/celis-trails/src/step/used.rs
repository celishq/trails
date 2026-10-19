//! CIP-06, "Uses": what one `uses` of a step names.

use crate::Step;

/// What one `uses` of a step names: a step it follows, or the id of a rule it uses.
#[derive(Clone, Copy)]
pub enum Used<'a> {
    Follow(Step<'a>),
    Rule(&'a str),
}
