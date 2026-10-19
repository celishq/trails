//! CIP-07, "Rules of a name": a trail that holds a name.

use crate::{Name, Step, Used};
use celis_nostr::Note;

/// A trail that holds a name, with the record that gave it the name.
#[derive(Clone, Copy)]
pub struct NamedTrail<'a> {
    pub name: &'a Name,
    pub trail: Step<'a>,
    pub record: &'a Note,
}

impl<'a> NamedTrail<'a> {
    /// Each named trail that a step on the `under` chain of `step` follows, and each named trail that a
    /// step of one found follows, at any depth, each once.
    pub fn followed(step: Step<'a>) -> Vec<NamedTrail<'a>> {
        let chain = std::iter::successors(Some(step), |s| s.under());
        let mut found: Vec<NamedTrail<'a>> = Vec::new();
        let mut next: Vec<NamedTrail<'a>> = chain.flat_map(Step::uses).filter_map(named).collect();
        while let Some(named_trail) = next.pop() {
            if found.iter().any(|f| f.trail.id() == named_trail.trail.id()) {
                continue;
            }
            let tree = named_trail.trail.tree().into_iter();
            next.extend(tree.flat_map(Step::uses).filter_map(named));
            found.push(named_trail);
        }
        found
    }
}

/// The named trail of the step `used` follows.
fn named(used: Used) -> Option<NamedTrail> {
    match used {
        Used::Follow(step) => step.trail().named(),
        Used::Rule(_) => None,
    }
}
