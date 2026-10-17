//! The whole id of a decision.

use serde::Serialize;
use std::fmt;
use std::ops::Deref;

/// The whole id of a decision record, as a link to it holds it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct DecisionId(String);

impl DecisionId {
    pub fn new(id: impl Into<String>) -> DecisionId {
        DecisionId(id.into())
    }
}

impl Deref for DecisionId {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DecisionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
