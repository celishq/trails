//! **The envelope** — the only thing that crosses a relay (NIP-01). A note is self-certifying: its `id` is
//! the sha256 of a compact array of the fields that matter, and its `sig` is a BIP-340 signature over that
//! id. Anyone can check both without asking anyone anything.
//!
//! The id covers `pubkey`, `created_at`, `kind`, `tags` and `content` — so it changes if any of them do, and
//! two clients that serialize the same note agree on it. `content` is opaque here: ciphertext for a log
//! entry, plain markdown for a public cell.

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
mod tests {
    use super::*;

    /// Two notes that have been on the network since 2022, signed by other clients. Nothing here was
    /// produced by this crate, so agreeing with them is a real check and not a round trip.
    const WILD_KIND_1: &str = r#"{"kind":1,"id":"dc90c95f09947507c1044e8f48bcf6350aa6bff1507dd4acfc755b9239b5c962","pubkey":"3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d","created_at":1644271588,"tags":[],"content":"now that https://blueskyweb.org/blog/2-7-2022-overview was announced we can stop working on nostr?","sig":"230e9d8f0ddaf7eb70b5f7741ccfa37e87a455c9a469282e3464e2052d3192cd63a167e196e381ef9d7e69e9ea43af2443b839974dc85d8aaab9efe1d9296524"}"#;

    /// Tags, and a content string full of quotes and slashes — the escaping the id depends on.
    const WILD_KIND_3: &str = r#"{"kind":3,"id":"9e662bdd7d8abc40b5b15ee1ff5e9320efc87e9274d8d440c58e6eed2dddfbe2","pubkey":"373ebe3d45ec91977296a178d9f19f326c70631d2a1b0bbba5c5ecc2eb53b9e7","created_at":1644844224,"tags":[["p","3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d"],["p","75fc5ac2487363293bd27fb0d14fb966477d0f1dbc6361d37806a6a740eda91e"],["p","46d0dfd3a724a302ca9175163bdf788f3606b3fd1bb12d5fe055d1e418cb60ea"]],"content":"{\"wss://nostr-pub.wellorder.net\":{\"read\":true,\"write\":true},\"wss://nostr.bitcoiner.social\":{\"read\":false,\"write\":true},\"wss://expensive-relay.fiatjaf.com\":{\"read\":true,\"write\":true},\"wss://relayer.fiatjaf.com\":{\"read\":true,\"write\":true},\"wss://relay.bitid.nz\":{\"read\":true,\"write\":true},\"wss://nostr.rocks\":{\"read\":true,\"write\":true}}","sig":"811355d3484d375df47581cb5d66bed05002c2978894098304f20b595e571b7e01b2efd906c5650080ffe49cf1c62b36715698e9d88b9e8be43029a2f3fa66be"}"#;

    fn wild(json: &str) -> Note {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn our_id_reproduces_the_id_another_client_gave_a_note() {
        for json in [WILD_KIND_1, WILD_KIND_3] {
            let note = wild(json);
            assert_eq!(note.digest(), note.id);
        }
    }

    #[test]
    fn a_note_from_the_wild_verifies() {
        for json in [WILD_KIND_1, WILD_KIND_3] {
            assert!(wild(json).verifies());
        }
    }

    #[test]
    fn editing_a_note_after_it_was_signed_breaks_it() {
        let note = wild(WILD_KIND_1);
        let edited = Note {
            content: "something else".into(),
            ..note.clone()
        };
        assert_ne!(edited.digest(), edited.id, "the id no longer describes it");
        assert!(!edited.verifies());

        let relabelled = Note {
            id: edited.digest(),
            ..edited
        };
        assert!(
            !relabelled.verifies(),
            "recomputing the id does not repair the signature"
        );
    }

    #[test]
    fn a_note_survives_a_json_round_trip_unchanged() {
        let note = wild(WILD_KIND_3);
        let restored: Note = serde_json::from_str(&serde_json::to_string(&note).unwrap()).unwrap();
        assert_eq!(restored, note);
        assert!(restored.verifies());
    }

    #[test]
    fn a_note_we_write_verifies_and_carries_its_tags() {
        let account = Account::new(&[9u8; 32]).unwrap();
        let note = Note::write(
            &account,
            35_000u16,
            1_700_000_000,
            vec![
                vec!["h".into(), "group-1".into()],
                vec!["d".into(), "entry-1".into()],
            ],
            "ciphertext",
        )
        .unwrap();

        assert!(note.verifies());
        assert_eq!(note.pubkey, account.pubkey());
        assert_eq!(note.tag("h"), Some("group-1"));
        assert_eq!(note.tag("d"), Some("entry-1"));
        assert_eq!(note.tag("p"), None);
    }

    #[test]
    fn the_id_covers_every_signed_field() {
        let account = Account::new(&[9u8; 32]).unwrap();
        let note = Note::write(&account, 1u16, 1_700_000_000, vec![], "hello").unwrap();
        let vary = |other: Note| assert_ne!(other.digest(), note.id);

        vary(Note {
            content: "hello ".into(),
            ..note.clone()
        });
        vary(Note {
            created_at: note.created_at + 1,
            ..note.clone()
        });
        vary(Note {
            kind: 2,
            ..note.clone()
        });
        vary(Note {
            tags: vec![vec!["d".into(), "x".into()]],
            ..note.clone()
        });
    }
}
