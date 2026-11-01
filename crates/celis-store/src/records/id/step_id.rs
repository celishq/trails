//! The whole id of a step.

use serde::Serialize;
use std::fmt;
use std::ops::Deref;

/// The whole id of a step record, as a link to it holds it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct StepId(String);

impl StepId {
    pub fn new(id: impl Into<String>) -> StepId {
        StepId(id.into())
    }
}

impl Deref for StepId {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StepId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
