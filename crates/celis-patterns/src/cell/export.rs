//! The records of an exported cell.

use crate::{Cell, Example, Membership, PatternRecord};
use anyhow::Result;
use celis_nostr::Note;
use celis_store::{ProofFile, Refusal};
use std::path::Path;

/// The records of an exported cell, as `cell.jsonl` holds them: its patterns, oldest first, and the
/// membership that names them (`spec/09-cells.md`, "Export").
#[derive(Debug)]
pub struct Export {
    pub cell: Cell,
    pub patterns: Vec<(Note, PatternRecord)>,
    pub membership: Note,
}

impl Export {
    /// The file of its records, one a line.
    pub const RECORDS: &'static str = "cell.jsonl";
    /// The file of its page for a person.
    pub const PAGE: &'static str = "cell.md";

    /// The export of `patterns`, oldest first, and the `membership` that names them.
    pub fn new(patterns: Vec<(Note, PatternRecord)>, membership: Note) -> Export {
        let cell = Membership::read(&membership).expect("a membership").cell;
        Export {
            cell,
            patterns,
            membership,
        }
    }

    /// Its records as `cell.jsonl` holds them: the patterns, then the membership.
    pub fn text(&self) -> String {
        let notes = self
            .patterns
            .iter()
            .map(|(n, _)| n)
            .chain([&self.membership]);
        let lines: Vec<String> = notes
            .map(|n| serde_json::to_string(n).expect("JSON"))
            .collect();
        lines.join("\n") + "\n"
    }

    /// The path of the blob of `file` in the folder: `blobs/<sha256>.<ext>`, with the extension of its
    /// Blossom URI.
    pub fn blob(file: &ProofFile) -> String {
        let url = file.url("");
        let named = url.trim_start_matches("blossom:");
        format!("blobs/{}", named.split('?').next().unwrap_or(named))
    }

    /// Writes it to `folder`: its records, the bytes `bytes` gives for each file, and `page`.
    pub fn write(
        &self,
        folder: &Path,
        page: &str,
        bytes: impl Fn(&ProofFile) -> Result<Vec<u8>>,
    ) -> Result<()> {
        let write = |name: &str, bytes: &[u8]| -> Result<()> {
            let path = folder.join(name);
            std::fs::create_dir_all(path.parent().expect("a file of the folder"))?;
            Ok(std::fs::write(path, bytes)?)
        };
        write(Export::RECORDS, self.text().as_bytes())?;
        for file in self.files() {
            write(&Export::blob(file), &bytes(file)?)?;
        }
        write(Export::PAGE, page.as_bytes())
    }

    /// The export `text` holds, one record a line; refuses it whole when a record breaks a rule.
    pub fn read(text: &str) -> std::result::Result<Export, Refusal> {
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
            } else if Membership::read(&note).is_some() {
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
        let Membership {
            cell,
            patterns: ids,
        } = Membership::read(membership).expect("a membership");
        for (note, _) in &patterns {
            if note.pubkey != membership.pubkey || !ids.contains(&note.id) {
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
