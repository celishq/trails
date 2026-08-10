//! **Nostr** — the wire the vault's log travels on. A [`Pubkey`] is who someone is, an [`Account`] is the
//! secret behind one of those, an [`Identity`] is the recovery phrase one such account is derived from, and
//! a [`Note`] is the one envelope that crosses a relay.
//!
//! One more type appears only inside an envelope: a [`Rumor`] is a message addressed to one person, which
//! reaches them wrapped so that a relay learns neither who sent it nor what it says (NIP-59).
//!
//! A [`Filter`] is what a subscription asks for and an [`Answer`] is what a relay says about one note
//! (NIP-01); [`Information`] is what it says about itself (NIP-11). None of the three knows how a socket is
//! held — that is `celis-network`'s.
//!
//! A [`Kind`] says what a note is. Two things here know celis, and no others do: the [`Identity`] derivation
//! is salted for this application, so the same phrase used elsewhere never yields these keys, and the kinds
//! this application minted for itself sit beside the ones the NIPs assign. Both are here because a kind is
//! one namespace however it was minted, and one salt is one derivation — split across crates, each becomes
//! several places to change and no single place to trust.
//!
//! **The crypto knows nothing about celis.** It is secp256k1, BIP-340, NIP-01, NIP-19, NIP-44 v2 and NIP-59,
//! and nothing about cells or logs — so every rule in it is checkable against someone else's implementation,
//! which is what its tests do.

mod account;
mod answer;
mod filter;
mod identity;
mod information;
mod kind;
mod nip44;
mod nip59;
mod note;
mod pubkey;
mod tags;

pub use account::Account;
pub use answer::Answer;
pub use filter::Filter;
pub use identity::Identity;
pub use information::{Information, Limitation, Retention};
pub use kind::Kind;
pub use nip59::Rumor;
pub use note::Note;
pub use pubkey::Pubkey;
pub use tags::Tags;
