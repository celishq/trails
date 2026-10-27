//! CIP-09: a cell, a named set of patterns of one account.

pub mod cell_store;
pub mod cells;
pub mod export;
pub mod layer;
pub mod mask;
pub mod membership;

use celis_nostr::Pubkey;

/// A cell: its name, and the account that signs its memberships.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Cell {
    pub name: String,
    pub author: Pubkey,
}

impl Cell {
    /// Whether `name` is a cell name: 1 to 64 characters of `a-z`, `0-9` and `-`, the first not `-`.
    pub fn is_name(name: &str) -> bool {
        let allowed = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-';
        (1..=64).contains(&name.len()) && !name.starts_with('-') && name.chars().all(allowed)
    }
}
