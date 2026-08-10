//! **What a relay says about itself** (NIP-11) — the document it answers a plain HTTP request with, on the
//! same URL a client opens a websocket to.
//!
//! Every field is optional, so an absent one is a relay that did not say rather than one that said no.

use serde::Deserialize;
use serde_json::Value;

/// A relay's own document.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Information {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub retention: Vec<Retention>,
    #[serde(default)]
    pub limitation: Limitation,
}

/// The limits a relay holds a client to. A caller that needs either may treat silence as a no — unlike the
/// kinds, which are only advised on.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Limitation {
    /// Whether this relay serves nothing until a client has answered its challenge (NIP-42).
    #[serde(default)]
    pub auth_required: bool,
    /// The most notes it answers one subscription at once, or `None` when it has not said.
    #[serde(default)]
    pub max_limit: Option<u32>,
}

/// One retention rule: how long the relay keeps the kinds it names. A rule may name a kind on its own or a
/// `[first, last]` range of them.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Retention {
    #[serde(default)]
    kinds: Vec<Value>,
}

impl Information {
    /// Whether this relay says it keeps notes of `kind` — true when it names the kind, and true when it
    /// names no kinds at all, since a relay that has not said has not refused.
    ///
    /// A number rather than a [`Kind`](crate::Kind): what a relay keeps is asked about kinds this build does
    /// not write, and a caller has to be able to ask.
    pub fn keeps(&self, kind: u16) -> bool {
        let mut named = self.named();
        named.peek().is_none() || named.any(|named| covers(named, kind))
    }

    /// The kinds this relay names, as it names them — what a refusal quotes back.
    pub fn kinds(&self) -> String {
        self.named()
            .map(|kind| kind.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }

    /// Every kind its retention rules name, across all of them.
    fn named(&self) -> std::iter::Peekable<impl Iterator<Item = &Value>> {
        self.retention
            .iter()
            .flat_map(|rule| &rule.kinds)
            .peekable()
    }
}

/// Whether a NIP-11 `kinds` entry covers `kind` — the number itself, or a `[first, last]` range containing
/// it.
fn covers(named: &Value, kind: u16) -> bool {
    match named {
        Value::Number(one) => one.as_u64() == Some(kind as u64),
        Value::Array(range) => {
            let at = |i: usize| range.get(i).and_then(Value::as_u64);
            matches!((at(0), at(1)), (Some(first), Some(last)) if (first..=last).contains(&(kind as u64)))
        }
        _ => false,
    }
}
