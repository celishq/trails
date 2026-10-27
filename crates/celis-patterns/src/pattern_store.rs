//! The patterns of a store, and the account's writes of them.

use crate::{Cell, Cells, PatternRecord, Patterns};
use celis_nostr::Note;
use celis_store::{Anchor, PatternId, ProofFile, Refusal, Store};
use std::collections::HashSet;

/// The patterns of a store, and the account's writes of them, each signed at the store's time.
pub trait PatternStore {
    fn patterns(&self) -> Patterns<'_>;

    /// Writes `pattern`, each `within` taken whole from a live pattern of any project; replacing the live
    /// pattern of its project that `replaces` names, and keeping its examples and `within`.
    fn decide(&self, pattern: PatternRecord, replaces: Option<&str>) -> Result<Note, Refusal>;

    /// Writes a membership that puts the live patterns of the account that `ids` name in its cell
    /// `name`; refuses a name that a cell of another account holds.
    fn join(&self, name: &str, ids: &[String]) -> Result<Note, Refusal>;

    /// Writes a mask that turns the cell `name` on or off in the project of `anchor`, or in every project
    /// with none; refuses a name that no cell or two cells hold.
    fn turn(&self, name: &str, on: bool, anchor: Option<&Anchor>) -> Result<Note, Refusal>;

    /// Removes the records of the cell `name` of another account, and each blob that only they name;
    /// how many records. Refuses a cell of the account, and a name that no cell holds.
    fn leave(&mut self, name: &str) -> Result<usize, Refusal>;
}

impl PatternStore for Store {
    fn patterns(&self) -> Patterns<'_> {
        Patterns::new(self.records())
    }

    fn decide(&self, mut pattern: PatternRecord, replaces: Option<&str>) -> Result<Note, Refusal> {
        let project = pattern.anchor.project.clone();
        let patterns = self.patterns();
        let within: Result<Vec<PatternId>, Refusal> = pattern
            .within
            .iter()
            .map(|id| Ok(PatternId::new(patterns.find(id)?.id())))
            .collect();
        pattern.within = within?;
        if let Some(id) = replaces {
            let old = patterns.view(&project).find(id)?.record;
            pattern.examples = pattern.examples.with(&old.examples);
            let kept = old.within.iter().filter(|w| !pattern.within.contains(w));
            let kept: Vec<PatternId> = kept.cloned().collect();
            pattern.within.extend(kept);
            pattern.replaces = Some(PatternId::new(patterns.view(&project).find(id)?.id()));
        }
        self.sign(|account, now| pattern.record(account, now))
    }

    fn join(&self, name: &str, ids: &[String]) -> Result<Note, Refusal> {
        if !Cell::is_name(name) {
            return Err(Refusal::Usage(format!(
                "a cell name is 1 to 64 characters of a-z, 0-9 and -, not {name}"
            )));
        }
        if ids.is_empty() {
            return Err(Refusal::Usage("cell add needs a pattern id".to_string()));
        }
        let author = self.author()?.unwrap_or_default();
        let cells = Cells::new(self.records());
        if cells.named(name).iter().any(|c| c.author != author) {
            return Err(Refusal::CellTaken(name.to_string()));
        }
        let patterns = self.patterns();
        let found: Result<Vec<&str>, Refusal> = ids
            .iter()
            .map(|id| {
                let pattern = patterns.find(id)?;
                match pattern.note.pubkey.hex() == author {
                    true => Ok(pattern.id()),
                    false => Err(Refusal::NoPattern(id.clone())),
                }
            })
            .collect();
        let found = found?;
        self.sign(|account, now| Cell::membership(account, now, name, &found))
    }

    fn turn(&self, name: &str, on: bool, anchor: Option<&Anchor>) -> Result<Note, Refusal> {
        let cells = Cells::new(self.records());
        let cell = match cells.named(name)[..] {
            [] => return Err(Refusal::NoCell(name.to_string())),
            [cell] => cell.clone(),
            _ => return Err(Refusal::CellTaken(name.to_string())),
        };
        let project = anchor.map(|a| a.project.as_str());
        let after = cells.masked_at(&cell, project).map(|at| at + 1);
        self.sign(|account, now| cell.mask(account, now.max(after.unwrap_or(0)), on, anchor))
    }

    fn leave(&mut self, name: &str) -> Result<usize, Refusal> {
        let author = self.author()?.unwrap_or_default();
        let patterns = self.patterns();
        let named = patterns.cells().named(name);
        let cell = match named.first() {
            None => return Err(Refusal::NoCell(name.to_string())),
            Some(_) if named.iter().any(|c| c.author == author) => {
                return Err(Refusal::CellYours(name.to_string()))
            }
            Some(cell) => (*cell).clone(),
        };
        let held = patterns.every().iter();
        let held = held.filter(|p| patterns.holding(p).contains(&&cell));
        let mut gone: HashSet<&str> = held.map(|p| p.id()).collect();
        let memberships = self
            .records()
            .iter()
            .filter(|n| Cell::read(n).is_some_and(|(c, _)| c == cell));
        gone.extend(memberships.map(|n| n.id.as_str()));
        let (records, kept): (Vec<&Note>, Vec<&Note>) = self
            .records()
            .iter()
            .partition(|n| gone.contains(n.id.as_str()));
        let named: HashSet<String> = kept.iter().flat_map(|n| blobs(n)).collect();
        let blobs: HashSet<String> = records.iter().flat_map(|n| blobs(n)).collect();
        let blobs: Vec<String> = blobs.difference(&named).cloned().collect();
        let records: Vec<Note> = records.into_iter().cloned().collect();
        self.remove(&records, &blobs)?;
        Ok(records.len())
    }
}

/// The sha256 of each file that a tag of `note` names (`spec/08-files.md`).
fn blobs(note: &Note) -> Vec<String> {
    let files = note
        .tags
        .iter()
        .filter_map(|t| ProofFile::read(t).or_else(|| ProofFile::blossom(t.get(1..)?)));
    files.map(|f| f.sha256).collect()
}
