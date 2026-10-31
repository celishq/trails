//! The envelope — the only thing that crosses a relay (NIP-01). A note is self-certifying: its `id` is the
//! sha256 of a compact array of the fields that matter, and its `sig` is a BIP-340 signature over that id.
//!
//! The id covers `pubkey`, `created_at`, `kind`, `tags` and `content`. `content` is opaque here: ciphertext
//! for a log entry, plain markdown for a public cell.

use crate::account::Account;
use crate::pubkey::Pubkey;
use anyhow::Result;
use data_encoding::HEXLOWER;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// One nostr event, in the shape it takes on the wire.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub pubkey: Pubkey,
    pub created_at: u64,
    pub kind: u16,
    pub tags: Vec<Vec<String>>,
    pub content: String,
    pub sig: String,
}

impl Note {
    /// Write and sign a note as `account`. `kind` takes a [`Kind`](crate::Kind) or the bare number a kind
    /// this build does not name travels as.
    pub fn write(
        account: &Account,
        kind: impl Into<u16>,
        created_at: u64,
        tags: impl Into<Vec<Vec<String>>>,
        content: impl Into<String>,
    ) -> Result<Self> {
        let mut note = Note {
            id: String::new(),
            pubkey: account.pubkey(),
            created_at,
            kind: kind.into(),
            tags: tags.into(),
            content: content.into(),
            sig: String::new(),
        };
        note.id = note.digest();
        note.sig = HEXLOWER.encode(&account.sign(&note.digest_bytes())?);
        Ok(note)
    }

    /// The id this note's own contents give it, whatever `id` currently says.
    pub fn digest(&self) -> String {
        HEXLOWER.encode(&self.digest_bytes())
    }

    /// Whether the note is genuine: the id is the hash of its own contents, and the signature is the
    /// author's over that id.
    pub fn verifies(&self) -> bool {
        let digest = self.digest_bytes();
        if HEXLOWER.encode(&digest) != self.id {
            return false;
        }
        let Ok(signature) = HEXLOWER.decode(self.sig.as_bytes()) else {
            return false;
        };
        self.pubkey.verifies(&digest, &signature)
    }

    /// The second after which this note is no longer to be served (NIP-40), if it says. A reader honours it
    /// too: a relay that keeps an expired note past its time is not a reason to act on one.
    pub fn expires_at(&self) -> Option<u64> {
        self.tag("expiration").and_then(|at| at.parse().ok())
    }

    /// The first value of the `name` tag, if the note carries one — `d` for an addressable note's address,
    /// `p` for the account a gift wrap is addressed to.
    pub fn tag(&self, name: &str) -> Option<&str> {
        self.slot(name, 1)
    }

    /// The first value of each `name` tag, in order.
    pub fn values<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        let tags = self.tags.iter();
        tags.filter(move |t| t.len() >= 2 && t[0] == name)
            .map(|t| t[1].as_str())
    }

    /// The values of its NIP-32 `l` tags in `namespace`, in order.
    pub fn labels<'a>(&'a self, namespace: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        let tags = self.tags.iter();
        let found = tags.filter(move |t| t.len() >= 3 && t[0] == "l" && t[2] == namespace);
        found.map(|t| t[1].as_str())
    }

    /// The value of its first NIP-32 `l` tag in `namespace`.
    pub fn label<'a>(&'a self, namespace: &'a str) -> Option<&'a str> {
        self.labels(namespace).next()
    }

    /// What a relay files this note under (NIP-01): an addressable kind (30000–39999) replaces on
    /// `pubkey:kind:d`, and everything else is kept as written under its own id.
    pub fn address(&self) -> String {
        match (30_000..40_000).contains(&self.kind) {
            true => format!(
                "{}:{}:{}",
                self.pubkey.hex(),
                self.kind,
                self.tag("d").unwrap_or_default()
            ),
            false => self.id.clone(),
        }
    }

    /// The second value of the `name` tag, if the note states one — NIP-29's role, beside the key a `p`
    /// names.
    pub fn second(&self, name: &str) -> Option<&str> {
        self.slot(name, 2)
    }

    fn slot(&self, name: &str, slot: usize) -> Option<&str> {
        self.tags
            .iter()
            .find(|tag| tag.first().is_some_and(|first| first == name))
            .and_then(|tag| tag.get(slot))
            .map(String::as_str)
    }

    fn digest_bytes(&self) -> [u8; 32] {
        let canonical = serde_json::json!([
            0,
            self.pubkey.hex(),
            self.created_at,
            self.kind,
            self.tags,
            self.content,
        ]);
        Sha256::digest(
            serde_json::to_string(&canonical)
                .expect("a note serializes")
                .as_bytes(),
        )
        .into()
    }
}

#[cfg(test)]
mod tests;
