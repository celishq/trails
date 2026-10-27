//! The records of an exported cell.

use crate::{Cell, Example, PatternRecord};
use celis_nostr::Note;
use celis_store::{ProofFile, Refusal};

/// The records of an exported cell, as `cell.jsonl` holds them: its patterns, oldest first, and the
/// membership that names them (`spec/09-cells.md`, "Export").
#[derive(Debug)]
pub struct Export {
    pub cell: Cell,
    pub patterns: Vec<(Note, PatternRecord)>,
    pub membership: Note,
}

impl Export {
    /// The export `text` holds, one record a line; refuses it whole when a record breaks a rule.
    pub fn read(text: &str) -> Result<Export, Refusal> {
        let invalid = |why: String| Refusal::Invalid(why);
        let mut patterns = vec![];
        let mut memberships = vec![];
        for (n, line) in text
            .lines()
            .enumerate()
            .filter(|(_, l)| !l.trim().is_empty())
        {
            let note: Note = serde_json::from_str(line)
                .map_err(|_| invalid(format!("line {} is not a record", n + 1)))?;
            if !note.verifies() {
                return Err(invalid(format!(
                    "the record on line {} does not verify",
                    n + 1
                )));
            }
            if let Some(record) = PatternRecord::read(&note) {
                patterns.push((note, record));
            } else if Cell::read(&note).is_some() {
                memberships.push(note);
            } else {
                return Err(invalid(format!(
                    "line {} is not a pattern or a membership",
                    n + 1
                )));
            }
        }
        let [membership] = &memberships[..] else {
            return Err(invalid(format!(
                "it holds {} memberships, not 1",
                memberships.len()
            )));
        };
        let (cell, ids) = Cell::read(membership).expect("a membership");
        for (note, _) in &patterns {
            if note.pubkey != membership.pubkey || !ids.contains(&note.id.as_str()) {
                return Err(invalid(format!(
                    "the cell does not hold the pattern {}",
                    note.id
                )));
            }
        }
        Ok(Export {
            cell,
            membership: membership.clone(),
            patterns,
        })
    }

    /// The file of each file example of its patterns.
    pub fn files(&self) -> impl Iterator<Item = &ProofFile> {
        let sides = self.patterns.iter().flat_map(|(_, r)| r.examples.sides());
        sides.filter_map(|(_, example)| match example {
            Example::File(file) => Some(file),
            _ => None,
        })
    }
}
