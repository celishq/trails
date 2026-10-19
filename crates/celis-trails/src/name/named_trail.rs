//! CIP-07, "Names": a trail that holds a name.

use crate::{Name, Step};
use celis_nostr::Note;

/// A trail that holds a name, with the record that gave it the name.
#[derive(Clone, Copy)]
pub struct NamedTrail<'a> {
    pub name: &'a Name,
    pub trail: Step<'a>,
    pub record: &'a Note,
}
