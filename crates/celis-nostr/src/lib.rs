//! celis-nostr — the nostr wire: the account key, signed notes, their encryption, and the relay they
//! travel through.
//!
//! Shapes: [`Pubkey`], [`Account`], [`Identity`], [`Nlink`], [`Note`], [`Rumor`], [`Filter`],
//! [`Answer`], [`Information`], [`Kind`]. Transport: [`Relay`]. Encryption happens here and nowhere else.

mod account;
mod answer;
pub mod authorization;
pub mod failure;
mod filter;
mod identity;
mod information;
mod kind;
pub mod mailbox;
mod nip44;
mod nip59;
mod nlink;
mod note;
mod pubkey;
pub mod relay;
mod tags;

pub use account::Account;
pub use answer::Answer;
pub use failure::{Failure, Verdict, Why};
pub use filter::Filter;
pub use identity::Identity;
pub use information::{Information, Limitation, Retention};
pub use kind::Kind;
pub use mailbox::{Held, Mailbox};
pub use nip59::Rumor;
pub use nlink::Nlink;
pub use note::Note;
pub use pubkey::Pubkey;
pub use relay::{Relay, SKEW};
pub use tags::Tags;

/// Unix seconds now — what a signed note stamps itself with, and what every host here checks against its own
/// clock. A clock that will not answer reads as the epoch, which every host refuses as far outside its
/// window: better a note turned down than one stamped a lie.
pub fn seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or(0)
}
