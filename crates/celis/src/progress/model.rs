//! What a tree, a report and the cards of stats are drawn from.

use super::Reuse;
use celis_trails::{Patterns, Steps, Trails};

/// The steps, patterns and reuse of a store at one moment.
pub struct Model<'a> {
    pub steps: &'a Steps<'a>,
    pub patterns: &'a Patterns<'a>,
    pub reuse: &'a Reuse<'a>,
}

impl Model<'_> {
    /// What `body` gives from the model of `trails`.
    pub fn of<T>(trails: &Trails, body: impl FnOnce(&Model) -> T) -> T {
        let (steps, patterns) = (trails.steps(), trails.patterns());
        let reuse = Reuse::new(&steps);
        body(&Model {
            steps: &steps,
            patterns: &patterns,
            reuse: &reuse,
        })
    }
}
