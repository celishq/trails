//! What a hosted route gave back instead of what was asked for.

use serde::{Deserialize, Serialize};
use std::fmt;

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

/// What became of one request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "why")]
pub enum Why {
    /// The host never answered — no route to it, DNS, TLS, or a timeout.
    Unreachable,
    /// The host answered, refusing, in the words it refused with.
    Refused { status: u16, body: String },
    /// The host answered, in a shape this build does not read.
    Unreadable { detail: String },
}

impl Failure {
    pub fn new(route: impl Into<String>, why: Why) -> Failure {
        Failure {
            route: route.into(),
            why,
        }
    }

    /// Whether the host answered about what was asked for, so there is nothing left to do about it. A host
    /// that never answered, or answered in a shape this build cannot read, was talking about itself, and the
    /// next round asks again.
    pub fn settled(&self) -> bool {
        matches!(self.why, Why::Refused { .. })
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.why {
            Why::Unreachable => write!(f, "{} could not be reached", self.route),
            Why::Refused { status, body } => write!(f, "{}: status {status}: {body}", self.route),
            Why::Unreadable { detail } => write!(f, "{} could not be read: {detail}", self.route),
        }
    }
}

impl std::error::Error for Failure {}
