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
mod tests {
    use super::*;

    /// What this system wraps: a record, which states its own kind (`spec/network.md`). The wrap layer
    /// never reads it, so any number would pass — this one is the one it actually sees.
    const KIND: u16 = Kind::Revision as u16;
    const AT: u64 = 1_785_326_400;

    fn account(byte: u8) -> Account {
        Account::new(&[byte; 32]).unwrap()
    }

    #[test]
    fn a_message_opens_for_its_reader_and_names_the_author_who_sealed_it() {
        let (author, reader) = (account(3), account(4));
        let note = Rumor::wrap(&author, &reader.pubkey(), KIND, AT, None, "the invite").unwrap();

        let rumor = Rumor::unwrap(&reader, &note).unwrap();
        assert_eq!(rumor.content, "the invite");
        assert_eq!(rumor.pubkey, author.pubkey());
        assert_eq!((rumor.kind, rumor.created_at), (KIND, AT));
    }

    #[test]
    fn nobody_else_opens_it_and_the_relay_sees_neither_party() {
        let (author, reader, stranger) = (account(3), account(4), account(5));
        let note = Rumor::wrap(&author, &reader.pubkey(), KIND, AT, None, "the invite").unwrap();

        assert!(Rumor::unwrap(&stranger, &note).is_err());
        assert!(
            Rumor::unwrap(&author, &note).is_err(),
            "not even the author reads it back — the carrier key is thrown away"
        );
        assert_ne!(
            note.pubkey,
            author.pubkey(),
            "a key minted for this message"
        );
        assert_eq!(
            note.tags,
            vec![vec!["p".to_string(), reader.pubkey().hex()]],
            "one tag, and no expiry — this message stands until it is answered"
        );
        assert!(note.verifies());
        assert!(!note.content.contains("the invite"));
    }

    #[test]
    fn the_same_message_wraps_differently_every_time() {
        let (author, reader) = (account(3), account(4));
        let once = Rumor::wrap(&author, &reader.pubkey(), KIND, AT, None, "same words").unwrap();
        let twice = Rumor::wrap(&author, &reader.pubkey(), KIND, AT, None, "same words").unwrap();

        assert_ne!(once.pubkey, twice.pubkey, "a fresh carrier each send");
        assert_ne!(once.id, twice.id);
        assert_eq!(
            Rumor::unwrap(&reader, &once).unwrap(),
            Rumor::unwrap(&reader, &twice).unwrap()
        );
    }

    /// The wrap `author` sends `reader` carrying `rumor` as it stands, sealed by `sealer` — what a test
    /// building a message by hand needs, since [`wrap`] mints its own rumor.
    fn wrapped(sealer: &Account, reader: &Account, rumor: &Rumor) -> Note {
        let seal = Note::write(
            sealer,
            Kind::Seal,
            AT,
            Vec::new(),
            sealer
                .encrypt(&reader.pubkey(), &serde_json::to_string(rumor).unwrap())
                .unwrap(),
        )
        .unwrap();
        let carrier = Account::generate().unwrap();
        Note::write(
            &carrier,
            Kind::Wrap,
            AT,
            vec![vec!["p".to_string(), reader.pubkey().hex()]],
            carrier
                .encrypt(&reader.pubkey(), &serde_json::to_string(&seal).unwrap())
                .unwrap(),
        )
        .unwrap()
    }

    #[test]
    /// A rumor is a regular event minus its signature (NIP-59), so it carries every field one does — and any
    /// client can read what this one writes.
    fn a_rumor_carries_every_field_a_nostr_event_does() {
        let (author, reader) = (account(3), account(4));
        let note = Rumor::wrap(&author, &reader.pubkey(), KIND, AT, None, "the invite").unwrap();

        let rumor = Rumor::unwrap(&reader, &note).unwrap();
        assert_eq!(rumor.id, rumor.digest(), "and its id is its own hash");
        assert_eq!(rumor.tags, Vec::<Vec<String>>::new());
        assert_eq!(
            serde_json::to_value(&rumor)
                .unwrap()
                .as_object()
                .unwrap()
                .keys()
                .cloned()
                .collect::<std::collections::BTreeSet<String>>(),
            ["content", "created_at", "id", "kind", "pubkey", "tags"]
                .iter()
                .map(|it| it.to_string())
                .collect()
        );
    }

    #[test]
    /// An id that does not describe the rumor beside it is refused, the same way a note's own is. Without
    /// the check the field would be decoration a sender could put anything in.
    fn a_rumor_whose_id_does_not_describe_it_is_refused() {
        let (author, reader) = (account(3), account(4));
        let mut rumor = Rumor {
            id: String::new(),
            pubkey: author.pubkey(),
            created_at: AT,
            kind: KIND,
            tags: Vec::new(),
            content: "the invite".to_string(),
        };
        rumor.id = rumor.digest();
        assert!(Rumor::unwrap(&reader, &wrapped(&author, &reader, &rumor)).is_ok());

        rumor.content.push('!');
        assert!(Rumor::unwrap(&reader, &wrapped(&author, &reader, &rumor)).is_err());
    }

    #[test]
    /// The seal's second is pushed back and the wrap's is not. A device resumes a walk from the wrap's
    /// second, so a wrap stamped in the past would fall behind a cursor already past it.
    fn the_seal_is_pushed_back_and_the_wrap_carries_the_send_second() {
        let (author, reader) = (account(3), account(4));
        let note = Rumor::wrap(&author, &reader.pubkey(), KIND, AT, None, "the invite").unwrap();

        assert_eq!(note.created_at, AT, "the wrap says when it was sent");
        assert_eq!(
            Rumor::unwrap(&reader, &note).unwrap().created_at,
            AT,
            "and the rumor says when the message is from"
        );

        let seals: std::collections::BTreeSet<u64> = (0..24)
            .map(|_| {
                let note =
                    Rumor::wrap(&author, &reader.pubkey(), KIND, AT, None, "the invite").unwrap();
                let seal: Note =
                    serde_json::from_str(&reader.decrypt(&note.pubkey, &note.content).unwrap())
                        .unwrap();
                seal.created_at
            })
            .collect();
        assert!(seals.len() > 1, "drawn fresh each time: {seals:?}");
        assert!(
            seals.iter().all(|at| (AT - TWEAK..=AT).contains(at)),
            "and never ahead, nor further back than the NIP suggests: {seals:?}"
        );
    }

    #[test]
    fn a_message_claiming_an_author_that_did_not_seal_it_is_refused() {
        let (liar, victim, reader) = (account(3), account(4), account(5));
        let mut rumor = Rumor {
            id: String::new(),
            pubkey: victim.pubkey(),
            created_at: AT,
            kind: KIND,
            tags: Vec::new(),
            content: "not mine".to_string(),
        };
        rumor.id = rumor.digest();
        assert!(Rumor::unwrap(&reader, &wrapped(&liar, &reader, &rumor)).is_err());
    }

    #[test]
    fn an_expiring_message_says_so_where_the_relay_can_read_it() {
        let (author, reader) = (account(3), account(4));
        let note = Rumor::wrap(
            &author,
            &reader.pubkey(),
            KIND,
            AT,
            Some(AT + 60),
            "the receipt",
        )
        .unwrap();

        assert_eq!(note.expires_at(), Some(AT + 60));
        assert!(note.verifies(), "the expiry is signed, not appended after");
        assert_eq!(
            Rumor::unwrap(&reader, &note).unwrap().content,
            "the receipt",
            "and it still opens the same way"
        );
    }

    #[test]
    fn a_tampered_wrap_and_a_note_of_another_kind_are_refused() {
        let (author, reader) = (account(3), account(4));
        let note = Rumor::wrap(&author, &reader.pubkey(), KIND, AT, None, "the invite").unwrap();

        let tampered = Note {
            content: "not the ciphertext it was".to_string(),
            ..note.clone()
        };
        assert!(Rumor::unwrap(&reader, &tampered).is_err());

        let plain = Note::write(&author, 1u16, AT, Vec::new(), "hello").unwrap();
        assert!(Rumor::unwrap(&reader, &plain).is_err());
    }
}
