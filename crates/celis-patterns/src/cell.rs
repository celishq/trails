//! CIP-09: a cell, a named set of patterns of one account.

/// A cell: its name, and the hex pubkey of the account that signs its memberships.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Cell {
    pub name: String,
    pub author: String,
}

impl Cell {
    /// Whether `name` is a cell name: 1 to 64 characters of `a-z`, `0-9` and `-`, the first not `-`.
    pub fn is_name(name: &str) -> bool {
        let allowed = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-';
        (1..=64).contains(&name.len()) && !name.starts_with('-') && name.chars().all(allowed)
    }
}
