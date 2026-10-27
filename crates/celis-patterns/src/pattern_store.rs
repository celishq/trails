//! The patterns of a store, and the account's writes of them.

use crate::{Cell, Cells, PatternRecord, Patterns};
use celis_nostr::Note;
use celis_store::{Anchor, PatternId, Refusal, Store};

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
            .map(|id| Ok(PatternId::new(patterns.find(None, id)?.id())))
            .collect();
        pattern.within = within?;
        if let Some(id) = replaces {
            let old = patterns.find(Some(&project), id)?.record;
            pattern.examples = pattern.examples.with(&old.examples);
            let kept = old.within.iter().filter(|w| !pattern.within.contains(w));
            let kept: Vec<PatternId> = kept.cloned().collect();
            pattern.within.extend(kept);
            pattern.replaces = Some(PatternId::new(patterns.find(Some(&project), id)?.id()));
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
                let pattern = patterns.find(None, id)?;
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
        self.sign(|account, now| cell.mask(account, now, on, anchor))
    }
}
