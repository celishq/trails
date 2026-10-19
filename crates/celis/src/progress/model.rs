//! What a tree, a report and the cards of stats are drawn from.

use super::Reuse;
use celis_trails::{Rules, Steps, Trails};

/// The steps, rules and reuse of a store at one moment.
pub struct Model<'a> {
    pub steps: &'a Steps<'a>,
    pub rules: &'a Rules<'a>,
    pub reuse: &'a Reuse<'a>,
}

impl Model<'_> {
    /// What `body` gives from the model of `trails`.
    pub fn of<T>(trails: &Trails, body: impl FnOnce(&Model) -> T) -> T {
        let (steps, rules) = (trails.steps(), trails.rules());
        let reuse = Reuse::new(&steps);
        body(&Model {
            steps: &steps,
            rules: &rules,
            reuse: &reuse,
        })
    }
}
