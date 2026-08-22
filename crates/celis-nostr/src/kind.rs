//! **What a note is** — every kind this application writes, in one enum.
//!
//! Two families sit here together. Most of these numbers are somebody else's: a NIP assigns them and every
//! nostr client already writes them. The rest are ours, minted for records no other client reads, and they
//! avoid every assignment we know of in the ranges they sit in.
//!
//! They share one enum because a kind is one namespace however it was minted, and two variants cannot claim
//! one number — the compiler refuses it. Split across crates, nothing but a test said the numbers still
//! agreed.

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The kind a note travels as (NIP-01).
///
/// The number is the wire form: [`Display`](std::fmt::Display) writes it, `u16` converts to it, and
/// [`of`](Kind::of) reads one back. A kind this build does not know has no variant, which is why
/// [`Note::kind`](crate::Note::kind) stays a `u16` — a record a newer build wrote still has to verify, sort
/// and reach the next device.
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// What an account says about itself (NIP-01) — the note every nostr client shows a person by, and the
    /// note a name is claimed with (`spec/public-profile.md`).
    Profile = 0,
    /// A take-back of the record at an address (NIP-09), with a `k` naming the kind that address is in.
    Deletion = 5,
    /// A yes or a no (NIP-25) — `+` or `-` in the content, and a `k` naming the kind it answers.
    Reaction = 7,
    /// The sealed layer inside a gift wrap (NIP-59), which never crosses a relay on its own.
    Seal = 13,
    /// A gift wrap (NIP-59) — the only one of the three layers a relay ever sees.
    Wrap = 1_059,
    /// A file's address (NIP-94): these bytes, this size, this type, at this url.
    Blob = 1_063,
    /// A comment on a cell (NIP-22). Addressed by the comment's own `d` so an edit supersedes it, which
    /// this build's fold decides rather than the number's range.
    Comment = 1_111,
    /// A relay authenticating a client on its socket (NIP-42).
    RelayAuth = 22_242,
    /// A call authorizing itself to a blob store (BUD-01).
    BlossomAuth = 24_242,
    /// A call authorizing itself over HTTP (NIP-98) — the method, the url, and the hash of the body.
    HttpAuth = 27_235,
    /// An invitation onto a server (NIP-43). Addressed by the invitation's own `d`, which pairs it with
    /// [`Leave`](Kind::Leave) below.
    ServerInvitation = 28_935,
    /// Somebody taking themselves off a server (NIP-43), signed by the account leaving.
    Leave = 28_936,
    /// Where to leave something addressed to an account (NIP-17).
    Inbox = 10_050,
    /// Where this account's public notes live (NIP-65) — the list a stranger follows from a name to an
    /// article.
    Relays = 10_002,
    /// A public cell as the world reads it (NIP-23) — plain markdown addressed by the cell's slug, which
    /// any nostr client renders without our client or our format.
    Article = 30_023,

    /// A cell as it was written — content, kept whole and forever, keyed by its own id.
    Revision = 8_100,
    /// The bytes at a hash are off the blob store. Pairs with [`Blob`](Kind::Blob): bytes are live until
    /// something says they stopped.
    BlobRemoved = 8_101,
    /// Where one account stands on one server, signed by the owner or an admin.
    ///
    /// **Kept, never replaced**, which is why it sits here rather than among the addressable kinds. Authority
    /// is read as the walk goes, so the grant that made an admin is what every record they wrote rests on —
    /// and a relay keeping only the newest record per member would drop that grant the moment they were taken
    /// off, unmaking everything they had done (`spec/network.md`).
    Membership = 8_102,
    /// The rumor a record travels inside a wrap as. What it carries is a record with a kind of its own.
    Record = 35_000,
    /// An invitation to read one cell, addressed by the invitation's own id. Nothing writes one now
    /// (`spec/log.md`); the number stays spent, as do 35001 and 35003 beside it.
    Invitation = 35_002,
    /// A server's own record: who owns it, where it answers, and who may leave something there.
    Server = 35_010,
    /// A server's sealing key and when it opened.
    Epoch = 35_012,
    /// The epoch private keys one member may read, sealed to them alone.
    Keyring = 35_013,
}

/// Every kind, in one list — what [`Kind::of`] reads a number back through. Kept here rather than as a
/// second `match` over the same numbers, so each one is written once.
const ALL: [Kind; 23] = [
    Kind::Profile,
    Kind::Deletion,
    Kind::Reaction,
    Kind::Seal,
    Kind::Wrap,
    Kind::Blob,
    Kind::Comment,
    Kind::Revision,
    Kind::BlobRemoved,
    Kind::Membership,
    Kind::RelayAuth,
    Kind::BlossomAuth,
    Kind::HttpAuth,
    Kind::ServerInvitation,
    Kind::Leave,
    Kind::Inbox,
    Kind::Relays,
    Kind::Article,
    Kind::Record,
    Kind::Invitation,
    Kind::Server,
    Kind::Epoch,
    Kind::Keyring,
];

impl Kind {
    /// The kind `number` names, or `None` for one this build does not know.
    pub fn of(number: u16) -> Option<Kind> {
        ALL.into_iter().find(|kind| u16::from(*kind) == number)
    }
}

impl From<Kind> for u16 {
    fn from(kind: Kind) -> u16 {
        kind as u16
    }
}

impl PartialEq<Kind> for u16 {
    /// Whether a kind off the wire is this one — what a reader asks of a [`Note::kind`](crate::Note::kind),
    /// which stays a number so an unknown one still travels.
    fn eq(&self, kind: &Kind) -> bool {
        *self == u16::from(*kind)
    }
}

impl PartialEq<u16> for Kind {
    fn eq(&self, number: &u16) -> bool {
        u16::from(*self) == *number
    }
}

impl Serialize for Kind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(u16::from(*self))
    }
}

impl<'de> Deserialize<'de> for Kind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let number = u16::deserialize(deserializer)?;
        Kind::of(number)
            .ok_or_else(|| D::Error::custom(format!("kind {number} is not one we know")))
    }
}

impl std::fmt::Display for Kind {
    /// The number, which is how a kind is written everywhere it appears as text — a `k` tag, an `a` tag's
    /// first field.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", u16::from(*self))
    }
}

#[cfg(test)]
mod tests;
