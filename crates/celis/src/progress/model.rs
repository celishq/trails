//! What a tree, a report and the cards of stats are drawn from.

use super::Reuse;
use celis_patterns::{PatternStore, Patterns};
use celis_store::Store;
use celis_trails::{Notes, Steps, TrailStore};

/// The steps, patterns, notes and reuse of a store at one moment.
#[derive(Clone, Copy)]
pub struct Model<'a> {
    pub steps: &'a Steps<'a>,
    pub patterns: &'a Patterns<'a>,
    pub notes: &'a Notes<'a>,
    pub reuse: &'a Reuse<'a>,
}

impl Model<'_> {
    /// What `body` gives from the model of `trails`.
    pub fn of<T>(trails: &Store, body: impl FnOnce(&Model) -> T) -> T {
        let (steps, patterns, notes) = (trails.steps(), trails.patterns(), trails.notes());
        let reuse = Reuse::new(&steps);
        body(&Model {
            steps: &steps,
            patterns: &patterns,
            notes: &notes,
            reuse: &reuse,
        })
    }
}
