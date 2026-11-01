//! The patterns of a store, and the account's writes of them.

use crate::{PatternRecord, Patterns};
use celis_nostr::Note;
use celis_store::{PatternId, Refusal, Store};

/// The patterns of a store, and the account's writes of them, each signed at the store's time.
pub trait PatternStore {
    fn patterns(&self) -> Patterns<'_>;

    /// Writes `pattern`, each `within` taken whole from a live pattern of any project; replacing the live
    /// pattern of its project that `replaces` names, and keeping its examples and `within`.
    fn write(&self, pattern: PatternRecord, replaces: Option<&str>) -> Result<Note, Refusal>;
}

impl PatternStore for Store {
    fn patterns(&self) -> Patterns<'_> {
        Patterns::new(self.records())
    }

    fn write(&self, mut pattern: PatternRecord, replaces: Option<&str>) -> Result<Note, Refusal> {
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
}
