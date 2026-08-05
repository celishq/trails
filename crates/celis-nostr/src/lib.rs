//! **Nostr** — the wire the vault's log travels on. Three types and nothing else: a [`Pubkey`] is who
//! someone is, an [`Account`] is the secret behind one of those, and a [`Note`] is the one envelope that
//! crosses a relay.
//!
//! A fourth type appears only inside an envelope: a [`Rumor`] is a message addressed to one person, which
//! reaches them wrapped so that a relay learns neither who sent it nor what it says (NIP-59).
//!
//! The crate is deliberately ignorant of celis. It knows secp256k1, BIP-340, NIP-01, NIP-19, NIP-44 v2 and
//! NIP-59, and nothing about cells or logs — so every rule here is checkable against someone else's
//! implementation, which is what its tests do.

mod account;
mod nip44;
mod nip59;
mod note;
mod pubkey;

pub use account::Account;
pub use nip59::{Rumor, WRAP_KIND};
pub use note::Note;
pub use pubkey::Pubkey;
