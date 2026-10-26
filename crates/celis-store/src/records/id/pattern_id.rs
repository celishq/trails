//! The whole id of a pattern.

use serde::Serialize;
use std::fmt;
use std::ops::Deref;

/// The whole id of a pattern record, as a link to it holds it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct PatternId(String);

impl PatternId {
    pub fn new(id: impl Into<String>) -> PatternId {
        PatternId(id.into())
    }
}

impl Deref for PatternId {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PatternId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
