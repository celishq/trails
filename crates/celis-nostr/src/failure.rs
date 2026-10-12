//! What a hosted route gave back instead of what was asked for.

use serde::{Deserialize, Serialize};
use std::fmt;

mod verdict;
mod why;

pub use verdict::Verdict;
pub use why::Why;

/// A hosted route that did not answer with what was asked for — which route, and what happened on it.
///
/// Attached to the `anyhow` chain a [`Gateway`](crate::Gateway) call fails with, so a caller tells these
/// apart without reading a message. The cause alone: what a person reads about it is composed where every
/// other sentence is, and the route is what says whether that sentence is about the model or the web.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Failure {
    /// The route as the gateway names it — `/ai/chat/completions`, `/search`, `/models`.
    pub route: String,
    pub why: Why,
}

impl Failure {
    pub fn new(route: impl Into<String>, why: Why) -> Failure {
        Failure {
            route: route.into(),
            why,
        }
    }

    /// What is left to do about this. A host that answered in a shape this build cannot read was talking
    /// about itself.
    pub fn verdict(&self) -> Verdict {
        match &self.why {
            Why::Unreachable => Verdict::Unreached,
            Why::Refused { status, after, .. } => Verdict::of_status(*status, *after),
            Why::Unreadable { .. } => Verdict::Again { after: None },
        }
    }

    /// Whether the host answered about what was asked for, so there is nothing left to do about it.
    pub fn settled(&self) -> bool {
        self.verdict() == Verdict::Settled
    }

    /// Whether the host answered that it does not hold what was asked for.
    pub fn missing(&self) -> bool {
        matches!(self.why, Why::Refused { status: 404, .. })
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.why {
            Why::Unreachable => write!(f, "{} could not be reached", self.route),
            Why::Refused { status, body, .. } => {
                write!(f, "{}: status {status}: {body}", self.route)
            }
            Why::Unreadable { detail } => write!(f, "{} could not be read: {detail}", self.route),
        }
    }
}

impl std::error::Error for Failure {}

#[cfg(test)]
mod tests;
