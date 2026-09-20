//! **What a subscription asks for** (NIP-01) — the shape a `REQ` frame carries, and nothing about the
//! socket it goes down.
//!
//! An absent field constrains nothing; an empty list matches nothing. Tags are a map because NIP-01 says
//! nothing about what a tag means: `#p` is the account a wrap is addressed to and `#h` the server a record
//! is about, but that is this application's reading of them rather than the protocol's.

use crate::kind::Kind;
use crate::note::Note;
use crate::pubkey::Pubkey;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::collections::BTreeMap;

/// One subscription's conditions. Every field is `AND`ed; within a list, any entry matches.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Filter {
    pub authors: Option<Vec<Pubkey>>,
    pub kinds: Option<Vec<Kind>>,
    pub since: Option<u64>,
    /// The newest note wanted. A relay answers newest-first, so walking this backwards is how a client reads
    /// past one page.
    pub until: Option<u64>,
    pub limit: Option<u32>,
    /// Tag name to the values wanted under it — what NIP-01 writes as `#p`, `#h`, and so on.
    pub tags: BTreeMap<char, Vec<String>>,
}

impl Filter {
    /// Every note of `kind`.
    pub fn of(kind: Kind) -> Self {
        Self {
            kinds: Some(vec![kind]),
            ..Self::default()
        }
    }

    /// The same conditions, also asking for notes carrying `name` with `value`. Asking twice under one name
    /// widens it: a relay answers a note matching any of the values.
    pub fn tagged(mut self, name: char, value: impl Into<String>) -> Self {
        self.tags.entry(name).or_default().push(value.into());
        self
    }

    /// Whether a relay answering this filter answers `note` (NIP-01): every stated condition holds, and a
    /// tag condition holds when any of the note's tags of that name carries any of the values asked for.
    pub fn admits(&self, note: &Note) -> bool {
        self.authors
            .as_ref()
            .is_none_or(|authors| authors.contains(&note.pubkey))
            && self
                .kinds
                .as_ref()
                .is_none_or(|kinds| kinds.iter().any(|kind| note.kind == *kind))
            && self.tags.iter().all(|(name, values)| {
                note.tags.iter().any(|tag| {
                    tag.first().map(String::as_str) == Some(&name.to_string())
                        && tag.get(1).is_some_and(|value| values.contains(value))
                })
            })
            && self.since.is_none_or(|since| note.created_at >= since)
            && self.until.is_none_or(|until| note.created_at <= until)
    }
}

impl Serialize for Filter {
    /// As NIP-01 wants it: an absent condition is absent from the frame rather than null, and each tag is
    /// its name behind a `#`.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let stated = [
            self.authors.is_some(),
            self.kinds.is_some(),
            self.since.is_some(),
            self.until.is_some(),
            self.limit.is_some(),
        ]
        .into_iter()
        .filter(|it| *it)
        .count();

        let mut map = serializer.serialize_map(Some(stated + self.tags.len()))?;
        if let Some(authors) = &self.authors {
            map.serialize_entry("authors", authors)?;
        }
        if let Some(kinds) = &self.kinds {
            map.serialize_entry("kinds", kinds)?;
        }
        if let Some(since) = &self.since {
            map.serialize_entry("since", since)?;
        }
        if let Some(until) = &self.until {
            map.serialize_entry("until", until)?;
        }
        if let Some(limit) = &self.limit {
            map.serialize_entry("limit", limit)?;
        }
        for (name, values) in &self.tags {
            map.serialize_entry(&format!("#{name}"), values)?;
        }
        map.end()
    }
}

#[cfg(test)]
mod tests;
