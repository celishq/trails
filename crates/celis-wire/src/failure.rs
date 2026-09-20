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
    /// The host answered, refusing, in the words it refused with. `after` is the seconds it said to wait
    /// before asking again, where it said any.
    Refused {
        status: u16,
        body: String,
        #[serde(default)]
        after: Option<u64>,
    },
    /// The host answered, in a shape this build does not read.
    Unreadable { detail: String },
}

/// What is left to do about a refusal — **the one place a status or a relay's word is read for it**, so
/// the gateway's retry, a push mark and a blob's record cannot read the same answer three ways.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// The host answered about what was asked for, and there is nothing left to do about it.
    Settled,
    /// The host answered about itself, so the next round asks again — after `after` seconds, where it said
    /// how long.
    Again { after: Option<u64> },
    /// The host never answered.
    Unreached,
}

/// The HTTP statuses a host answers about itself with: busy, over its rate, or failing.
const ABOUT_ITSELF: [u16; 5] = [429, 500, 502, 503, 504];

/// The words a relay writes about the note itself, so there is nothing left to do about it (NIP-01).
/// `invalid:` is malformed or superseded, `restricted:` is a permission, and `pow:` asks for work this build
/// will never do. `blocked:` is not among them: this network writes it for an account over its allowance,
/// which is a door that opens again (`spec/network.md`).
const FINAL: [&str; 3] = ["invalid:", "restricted:", "pow:"];

impl Verdict {
    /// The verdict an HTTP status carries, with `after` as the seconds a `Retry-After` named.
    pub fn of_status(status: u16, after: Option<u64>) -> Verdict {
        match ABOUT_ITSELF.contains(&status) {
            true => Verdict::Again { after },
            false => Verdict::Settled,
        }
    }

    /// The verdict a relay's refusal carries, read off its NIP-01 prefix and nothing else. A word this
    /// build cannot place, and no word at all, is the relay talking about itself (`spec/network.md`).
    pub fn of_word(reason: &str) -> Verdict {
        match FINAL.iter().any(|word| reason.starts_with(word)) {
            true => Verdict::Settled,
            false => Verdict::Again { after: None },
        }
    }
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
