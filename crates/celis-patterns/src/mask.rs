//! CIP-09, "Masks": a label that turns a cell on or off.

use crate::{Cell, Scope};
use anyhow::Result;
use celis_nostr::{Account, Note};
use celis_store::Anchor;

/// A mask: the cell it turns on or off, its project, none for every project, and when it was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mask {
    pub cell: Cell,
    pub on: bool,
    pub anchor: Option<Anchor>,
    pub at: u64,
}

impl Mask {
    /// This mask as a record that `signer` signs at its time.
    pub fn record(&self, signer: &Account) -> Result<Note> {
        let state = if self.on { "on" } else { "off" };
        let mut tags = vec![
            vec!["L".to_string(), NAMESPACE.to_string()],
            vec!["l".to_string(), state.to_string(), NAMESPACE.to_string()],
            vec!["cell".to_string(), self.cell.name.clone()],
            vec!["p".to_string(), self.cell.author.clone()],
        ];
        match &self.anchor {
            Some(anchor) => tags.extend(anchor.tags()),
            None => tags.push(vec!["scope".to_string(), Scope::Me.text()]),
        }
        Note::write(signer, 1985u16, self.at, tags, "")
    }

    /// The mask `note` is; `None` when it is none.
    pub fn read(note: &Note) -> Option<Mask> {
        let label = note
            .tags
            .iter()
            .find(|t| t.len() >= 3 && t[0] == "l" && t[2] == NAMESPACE)?;
        let on = match label[1].as_str() {
            "on" => true,
            "off" => false,
            _ => return None,
        };
        let cell = Cell {
            name: note.tag("cell")?.to_string(),
            author: note.tag("p")?.to_string(),
        };
        let anchor = match note.tag("scope").and_then(Scope::parse) {
            Some(Scope::Me) => None,
            _ => Some(Anchor::read(note)?),
        };
        Some(Mask {
            cell,
            on,
            anchor,
            at: note.created_at,
        })
    }

    /// Where it holds: in its project, or in every project.
    pub fn scope(&self) -> Scope {
        match self.anchor {
            Some(_) => Scope::Project,
            None => Scope::Me,
        }
    }

    /// The project id it holds in; none for every project.
    pub fn project(&self) -> Option<&str> {
        self.anchor.as_ref().map(|a| a.project.as_str())
    }
}

/// The label namespace of a mask.
const NAMESPACE: &str = "celis.mask";
