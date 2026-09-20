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
mod tests {
    use super::*;

    #[test]
    /// A refusal of the request is settled; a host that is busy, over its rate, or failing is asking to be
    /// asked again, and says how long where it can.
    fn a_status_says_what_is_left_to_do() {
        assert_eq!(Verdict::of_status(400, None), Verdict::Settled);
        assert_eq!(Verdict::of_status(402, None), Verdict::Settled);
        assert_eq!(Verdict::of_status(404, None), Verdict::Settled);
        assert_eq!(
            Verdict::of_status(429, Some(7)),
            Verdict::Again { after: Some(7) }
        );
        for status in [500, 502, 503, 504] {
            assert_eq!(
                Verdict::of_status(status, None),
                Verdict::Again { after: None },
                "{status}"
            );
        }
    }

    #[test]
    /// The prefix is the whole of the reading (`spec/fixtures/refusals.json`): three words are about the
    /// note, and every other word, a word inside the sentence, or no word is the relay about itself.
    fn a_relays_word_says_what_is_left_to_do() {
        for word in [
            "invalid: bad sig",
            "restricted: not a member",
            "pow: 20 bits",
        ] {
            assert_eq!(Verdict::of_word(word), Verdict::Settled, "{word}");
        }
        for word in [
            "blocked: over allowance",
            "rate-limited: slow down",
            "error: could not save: invalid: about another note",
            "something new",
            "",
        ] {
            assert_eq!(
                Verdict::of_word(word),
                Verdict::Again { after: None },
                "{word}"
            );
        }
    }

    /// Every prefix in `spec/fixtures/refusals.json`, which the host writes against — so the table here
    /// and the words the host writes cannot drift apart without failing. `took` is not a verdict: a relay
    /// that has the note refused nothing, and what reads that is the offer.
    #[test]
    fn every_prefix_the_host_writes_carries_the_verdict_the_fixture_states() {
        let stated: serde_json::Value =
            serde_json::from_str(include_str!("../../../spec/fixtures/refusals.json")).unwrap();
        let prefixes = stated["prefixes"].as_array().unwrap();
        assert!(!prefixes.is_empty());
        for stated in prefixes {
            let prefix = stated["prefix"].as_str().unwrap();
            let expected = match stated["offer"].as_str().unwrap() {
                "took" => continue,
                "refused" => Verdict::Settled,
                "unsettled" => Verdict::Again { after: None },
                other => panic!("the fixture names an offer this build does not read: {other}"),
            };
            assert_eq!(
                Verdict::of_word(&format!("{prefix}: what the host said")),
                expected,
                "`{prefix}:`"
            );
        }
    }

    #[test]
    fn a_failure_reads_its_verdict_off_what_became_of_it() {
        assert_eq!(
            Failure::new("/ai", Why::Unreachable).verdict(),
            Verdict::Unreached
        );
        assert_eq!(
            Failure::new(
                "/ai",
                Why::Unreadable {
                    detail: "not json".into()
                }
            )
            .verdict(),
            Verdict::Again { after: None }
        );
        let spent = Failure::new(
            "/ai",
            Why::Refused {
                status: 402,
                body: String::new(),
                after: None,
            },
        );
        assert!(spent.settled());
        let busy = Failure::new(
            "/ai",
            Why::Refused {
                status: 503,
                body: String::new(),
                after: Some(3),
            },
        );
        assert!(!busy.settled());
        assert_eq!(busy.verdict(), Verdict::Again { after: Some(3) });
    }
}
