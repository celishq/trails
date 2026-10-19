//! The whole id of a rule.

use serde::Serialize;
use std::fmt;
use std::ops::Deref;

/// The whole id of a rule record, as a link to it holds it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct RuleId(String);

impl RuleId {
    pub fn new(id: impl Into<String>) -> RuleId {
        RuleId(id.into())
    }
}

impl Deref for RuleId {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
