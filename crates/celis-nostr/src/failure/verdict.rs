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
    /// The host is still answering this same call, so the same call asks again after `after` seconds and
    /// is answered with it.
    Answering { after: u64 },
}

/// The HTTP statuses a host answers about itself with: busy, over its rate, or failing.
const ABOUT_ITSELF: [u16; 5] = [429, 500, 502, 503, 504];

/// The prefixes a relay writes about the note itself, nothing left to do about it (NIP-01). `blocked:` is
/// not among them: it is written for an account over its allowance, a door that opens again.
const FINAL: [&str; 3] = ["invalid:", "restricted:", "pow:"];

impl Verdict {
    /// The verdict an HTTP status carries, with `after` as the seconds a `Retry-After` named.
    pub fn of_status(status: u16, after: Option<u64>) -> Verdict {
        match (status, after) {
            (409, Some(after)) => Verdict::Answering { after },
            _ if ABOUT_ITSELF.contains(&status) => Verdict::Again { after },
            _ => Verdict::Settled,
        }
    }

    /// The verdict a relay's refusal carries, read off its NIP-01 prefix and nothing else. A word this
    /// build cannot place, and no word at all, is the relay talking about itself.
    pub fn of_word(reason: &str) -> Verdict {
        match FINAL.iter().any(|word| reason.starts_with(word)) {
            true => Verdict::Settled,
            false => Verdict::Again { after: None },
        }
    }
}
