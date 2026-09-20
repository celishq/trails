//! **Gift wrap** (NIP-59) — how one message reaches one person, with the relay learning neither who sent it
//! nor what it says.
//!
//! ```text
//!   rumor   what the author wrote, unsigned          — so a leaked copy is not a publishable event
//!   seal    the rumor, encrypted to the reader and signed by the author  — this is what proves authorship
//!   wrap    the seal, encrypted to the reader and signed by a key minted for this one message
//! ```
//!
//! Only the wrap crosses the relay. Its author is a throwaway key, so the relay sees a note from nobody to a
//! `p` tag; only the reader can open it, and only then does the author's signature appear.
//!
//! **The seal's second is pushed back, the wrap's is not.** NIP-59 spreads the layers so they cannot be
//! correlated by second. The wrap's second is what a device resumes a walk from
//! (`celis-runtime/src/network/entries.rs`), so pushing it back would leave a record published now, stamped
//! yesterday, behind a cursor that has already passed it — and nothing would ever fetch it again.

use crate::account::Account;
use crate::kind::Kind;
use crate::note::Note;
use crate::pubkey::Pubkey;
use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

/// How far back a seal's second may be pushed — NIP-59's own suggestion. The seal never crosses a relay on
/// its own, so nothing indexes or resumes from this second.
const TWEAK: u64 = 2 * 24 * 60 * 60;

/// The message inside a gift wrap: a regular event minus its signature (NIP-59).
///
/// Never signed, deliberately — a signature would make the message an event anyone could publish. The
/// author's `pubkey` here is trusted only because `unwrap` has checked it against the seal's signature, and
/// the `id` because `unwrap` has hashed the rest and found the same.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rumor {
    pub id: String,
    pub pubkey: Pubkey,
    pub created_at: u64,
    pub kind: u16,
    pub tags: Vec<Vec<String>>,
    pub content: String,
}

impl Rumor {
    /// The id these contents give it — NIP-01's canonical hash, which a rumor takes like any other event
    /// because it is one, short of the signature.
    fn digest(&self) -> String {
        Note {
            id: String::new(),
            pubkey: self.pubkey,
            created_at: self.created_at,
            kind: self.kind,
            tags: self.tags.clone(),
            content: self.content.clone(),
            sig: String::new(),
        }
        .digest()
    }

    /// Wrap `content` as `author` so that only `reader` can open it, and only they learn who sent it.
    /// `kind` says which of the author's messages this is; it stays inside the ciphertext. `expires_at`
    /// asks the relay to stop serving the wrap after that second (NIP-40) — the one thing about a message
    /// a relay is told, and it goes on the wrap because the layers inside are opaque to it. `None` for a
    /// message that stands until it is answered.
    pub fn wrap(
        author: &Account,
        reader: &Pubkey,
        kind: impl Into<u16>,
        created_at: u64,
        expires_at: Option<u64>,
        content: &str,
    ) -> Result<Note> {
        let mut rumor = Rumor {
            id: String::new(),
            pubkey: author.pubkey(),
            created_at,
            kind: kind.into(),
            tags: Vec::new(),
            content: content.to_string(),
        };
        rumor.id = rumor.digest();
        let seal = Note::write(
            author,
            Kind::Seal,
            created_at.saturating_sub(tweak()?),
            Vec::new(),
            author.encrypt(reader, &serde_json::to_string(&rumor)?)?,
        )?;
        let carrier = Account::generate()?;
        let mut tags = vec![vec!["p".to_string(), reader.hex()]];
        if let Some(at) = expires_at {
            tags.push(vec!["expiration".to_string(), at.to_string()]);
        }
        Note::write(
            &carrier,
            Kind::Wrap,
            created_at,
            tags,
            carrier.encrypt(reader, &serde_json::to_string(&seal)?)?,
        )
    }

    /// Open a gift wrap addressed to `reader`, answering what it carries and the author it proves. Refuses
    /// a wrong kind at either layer, a signature that does not hold, a payload this key cannot open, or a
    /// rumor claiming an author who did not seal it.
    pub fn unwrap(reader: &Account, wrap: &Note) -> Result<Rumor> {
        if wrap.kind != Kind::Wrap {
            bail!("kind {} is not a gift wrap", wrap.kind);
        }
        if !wrap.verifies() {
            bail!("the wrapper is not signed by the key that carried it");
        }
        let seal: Note = serde_json::from_str(&reader.decrypt(&wrap.pubkey, &wrap.content)?)?;
        if seal.kind != Kind::Seal {
            bail!("kind {} is not a seal", seal.kind);
        }
        if !seal.verifies() {
            bail!("the seal is not signed by the account that claims it");
        }
        let rumor: Rumor = serde_json::from_str(&reader.decrypt(&seal.pubkey, &seal.content)?)?;
        if rumor.pubkey != seal.pubkey {
            bail!("the message claims an author that did not seal it");
        }
        if rumor.id != rumor.digest() {
            bail!("the message's id does not describe it");
        }
        Ok(rumor)
    }
}

/// How far back to push one seal's second — anywhere in [`TWEAK`], drawn fresh for each message so two
/// sealed in one second do not land on one.
fn tweak() -> Result<u64> {
    let mut drawn = [0u8; 8];
    getrandom::getrandom(&mut drawn).map_err(|why| anyhow!("no randomness: {why}"))?;
    Ok(u64::from_le_bytes(drawn) % (TWEAK + 1))
}

#[cfg(test)]
mod tests;
