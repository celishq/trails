//! How sure a pattern is, from the steps that use it.

use celis_trails::Steps;
use std::collections::{HashMap, HashSet};

/// The steps that use a pattern before it has a level.
const USES: usize = 3;

/// The projects whose steps use a pattern before it has two stars.
const PROJECTS: usize = 2;

/// The level of each pattern that steps use, by its id (CIP-02, "Levels").
pub(crate) struct Level {
    marks: HashMap<String, &'static str>,
}

impl Level {
    /// The levels that the uses and the misled of `steps` give.
    pub fn of<'a>(steps: &'a Steps<'a>) -> Level {
        let mut uses: HashMap<&str, (usize, HashSet<&str>)> = HashMap::new();
        let mut misled = HashSet::new();
        for step in steps.all() {
            for id in step.uses() {
                let (n, projects) = uses.entry(id).or_default();
                *n += 1;
                projects.insert(step.record().anchor.project.as_str());
            }
            misled.extend(step.misled().into_iter().map(|(id, _)| id));
        }
        let marks = uses
            .into_iter()
            .filter(|(id, (n, _))| *n >= USES && !misled.contains(id))
            .map(|(id, (_, projects))| {
                let mark = if projects.len() >= PROJECTS {
                    "**"
                } else {
                    "*"
                };
                (id.to_string(), mark)
            })
            .collect();
        Level { marks }
    }

    /// The mark of the pattern `id`: `*`, `**`, or `None` when it has no level.
    pub fn mark(&self, id: &str) -> Option<&'static str> {
        self.marks.get(id).copied()
    }
}
