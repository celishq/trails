//! CIP-09, "Membership": a label that puts patterns in a cell.

use crate::Cell;
use anyhow::Result;
use celis_nostr::{Account, Note};

/// A membership: the cell, and the ids of the patterns it puts in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Membership {
    pub cell: Cell,
    pub patterns: Vec<String>,
}

impl Membership {
    /// The membership that puts the patterns `ids` in the cell `name`, signed by `signer` at `at`.
    pub fn write(signer: &Account, at: u64, name: &str, ids: &[&str]) -> Result<Note> {
        let mut tags = vec![
            vec!["L".to_string(), NAMESPACE.to_string()],
            vec!["l".to_string(), name.to_string(), NAMESPACE.to_string()],
        ];
        tags.extend(ids.iter().map(|id| vec!["e".to_string(), id.to_string()]));
        Note::write(signer, 1985u16, at, tags, "")
    }

    /// The membership `note` is; `None` when it is none.
    pub fn read(note: &Note) -> Option<Membership> {
        let name = note.label(NAMESPACE)?;
        if note.kind != 1985 || !Cell::is_name(name) {
            return None;
        }
        let cell = Cell {
            name: name.to_string(),
            author: note.pubkey,
        };
        let ids = note.tags.iter().filter(|t| t.len() >= 2 && t[0] == "e");
        let patterns = ids.map(|t| t[1].clone()).collect();
        Some(Membership { cell, patterns })
    }
}

/// The label namespace of a membership.
const NAMESPACE: &str = "celis.cell";
