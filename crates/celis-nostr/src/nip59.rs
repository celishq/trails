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
//! **The timestamps are not backdated.** NIP-59 suggests spreading them so the layers cannot be correlated
//! by second; every layer here carries the same second as the message it wraps.

use crate::account::Account;
use crate::kind::Kind;
use crate::note::Note;
use crate::pubkey::Pubkey;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// The message inside a gift wrap: what its author wrote, and who they are.
///
/// Never signed, deliberately — a signature would make the message an event anyone could publish. The
/// author's `pubkey` here is trusted only because `unwrap` has checked it against the seal's signature.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rumor {
    pub pubkey: Pubkey,
    pub created_at: u64,
    pub kind: u16,
    pub content: String,
}

/// Wrap `content` as `author` so that only `reader` can open it. `expires_at` asks the relay to stop serving
/// the wrap after that second (NIP-40) — the one thing about a message a relay is told, and it goes on the
/// wrap because the layers inside are opaque to it. `None` for a message that stands until it is answered.
pub fn wrap(
    author: &Account,
    reader: &Pubkey,
    kind: u16,
    created_at: u64,
    expires_at: Option<u64>,
    content: &str,
) -> Result<Note> {
    let rumor = Rumor {
        pubkey: author.pubkey(),
        created_at,
        kind,
        content: content.to_string(),
    };
    let seal = Note::write(
        author,
        Kind::Seal,
        created_at,
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

/// Open a gift wrap addressed to `reader`, answering what it carries and who wrote it. Refuses a wrong kind
/// at either layer, a signature that does not hold, a payload this key cannot open, or a rumor claiming an
/// author who did not seal it.
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
    Ok(rumor)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What this system wraps: a record, whatever kind it carries inside (`spec/network.md`). The wrap
    /// layer never reads it, so any number would pass — this one is the one it actually sees.
    const KIND: u16 = Kind::Record as u16;
    const AT: u64 = 1_785_326_400;

    fn account(byte: u8) -> Account {
        Account::new(&[byte; 32]).unwrap()
    }

    #[test]
    fn a_message_opens_for_its_reader_and_names_the_author_who_sealed_it() {
        let (author, reader) = (account(3), account(4));
        let note = author
            .wrap(&reader.pubkey(), KIND, AT, None, "the invite")
            .unwrap();

        let rumor = reader.unwrap(&note).unwrap();
        assert_eq!(rumor.content, "the invite");
        assert_eq!(rumor.pubkey, author.pubkey());
        assert_eq!((rumor.kind, rumor.created_at), (KIND, AT));
    }

    #[test]
    fn nobody_else_opens_it_and_the_relay_sees_neither_party() {
        let (author, reader, stranger) = (account(3), account(4), account(5));
        let note = author
            .wrap(&reader.pubkey(), KIND, AT, None, "the invite")
            .unwrap();

        assert!(stranger.unwrap(&note).is_err());
        assert!(
            author.unwrap(&note).is_err(),
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
        let once = author
            .wrap(&reader.pubkey(), KIND, AT, None, "same words")
            .unwrap();
        let twice = author
            .wrap(&reader.pubkey(), KIND, AT, None, "same words")
            .unwrap();

        assert_ne!(once.pubkey, twice.pubkey, "a fresh carrier each send");
        assert_ne!(once.id, twice.id);
        assert_eq!(
            reader.unwrap(&once).unwrap(),
            reader.unwrap(&twice).unwrap()
        );
    }

    #[test]
    fn a_message_claiming_an_author_that_did_not_seal_it_is_refused() {
        let (liar, victim, reader) = (account(3), account(4), account(5));
        let rumor = Rumor {
            pubkey: victim.pubkey(),
            created_at: AT,
            kind: KIND,
            content: "not mine".to_string(),
        };
        let seal = Note::write(
            &liar,
            Kind::Seal,
            AT,
            Vec::new(),
            liar.encrypt(&reader.pubkey(), &serde_json::to_string(&rumor).unwrap())
                .unwrap(),
        )
        .unwrap();
        let carrier = Account::generate().unwrap();
        let note = Note::write(
            &carrier,
            Kind::Wrap,
            AT,
            vec![vec!["p".to_string(), reader.pubkey().hex()]],
            carrier
                .encrypt(&reader.pubkey(), &serde_json::to_string(&seal).unwrap())
                .unwrap(),
        )
        .unwrap();

        assert!(reader.unwrap(&note).is_err());
    }

    #[test]
    fn an_expiring_message_says_so_where_the_relay_can_read_it() {
        let (author, reader) = (account(3), account(4));
        let note = author
            .wrap(&reader.pubkey(), KIND, AT, Some(AT + 60), "the receipt")
            .unwrap();

        assert_eq!(note.expires_at(), Some(AT + 60));
        assert!(note.verifies(), "the expiry is signed, not appended after");
        assert_eq!(
            reader.unwrap(&note).unwrap().content,
            "the receipt",
            "and it still opens the same way"
        );
    }

    #[test]
    fn a_tampered_wrap_and_a_note_of_another_kind_are_refused() {
        let (author, reader) = (account(3), account(4));
        let note = author
            .wrap(&reader.pubkey(), KIND, AT, None, "the invite")
            .unwrap();

        let tampered = Note {
            content: "not the ciphertext it was".to_string(),
            ..note.clone()
        };
        assert!(reader.unwrap(&tampered).is_err());

        let plain = Note::write(&author, 1u16, AT, Vec::new(), "hello").unwrap();
        assert!(reader.unwrap(&plain).is_err());
    }
}
