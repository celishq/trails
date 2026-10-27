//! CIP-09: a cell, a named set of patterns of one account.

use anyhow::Result;
use celis_nostr::{Account, Note};

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

    /// The membership that puts the patterns `ids` in the cell `name`, signed by `signer` at `at`.
    pub fn membership(signer: &Account, at: u64, name: &str, ids: &[&str]) -> Result<Note> {
        let mut tags = vec![
            vec!["L".to_string(), NAMESPACE.to_string()],
            vec!["l".to_string(), name.to_string(), NAMESPACE.to_string()],
        ];
        tags.extend(ids.iter().map(|id| vec!["e".to_string(), id.to_string()]));
        Note::write(signer, 1985u16, at, tags, "")
    }

    /// The cell `note` is a membership of, and the ids it names; `None` when it is no membership.
    pub fn read(note: &Note) -> Option<(Cell, Vec<&str>)> {
        let label = note
            .tags
            .iter()
            .find(|t| t.len() >= 3 && t[0] == "l" && t[2] == NAMESPACE)?;
        if note.kind != 1985 || !Cell::is_name(&label[1]) {
            return None;
        }
        let cell = Cell {
            name: label[1].clone(),
            author: note.pubkey.hex(),
        };
        let ids = note.tags.iter().filter(|t| t.len() >= 2 && t[0] == "e");
        Some((cell, ids.map(|t| t[1].as_str()).collect()))
    }
}

/// The label namespace of a membership.
const NAMESPACE: &str = "celis.cell";
