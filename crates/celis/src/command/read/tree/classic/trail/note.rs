//! The notes under a row: its text, proof, change, time and reason; and the patterns it uses.

use crate::command::read::tree::classic::row::Row;
use crate::command::read::tree::classic::Tree;
use crate::command::read::tree::details;
use crate::terminal::canvas::{Styled, DIM};
use celis_nostr::Note;
use celis_trails::Step;

impl Tree<'_> {
    /// Adds under `note` the `details` of its text, its proof, change and with `grade`
    /// its grade when done, or its reason when dropped, after `rail`.
    pub(in crate::command::read::tree::classic) fn closing(
        &self,
        note: Step,
        rail: &Styled,
        grade: bool,
        out: &mut Vec<Row>,
    ) {
        for (label, text) in details(note.record().detail()) {
            self.labelled(out, rail, label, text);
        }
        if let (Some(done), Some(line)) = (note.done_record(), crate::text::line::grade(note)) {
            self.labelled(out, rail, "done", &done.content);
            if let Some(changed) = done.tag("changed") {
                self.labelled(out, rail, "changed", changed);
            }
            if grade {
                self.labelled(out, rail, "took", line.trim_start_matches("took "));
            }
        }
        if let Some(reason) = note.dropped() {
            self.labelled(out, rail, "dropped", &reason.content);
        }
    }

    /// The patterns `note` uses.
    pub(in crate::command::read::tree::classic) fn uses(&self, note: Step) -> Vec<&Note> {
        let found = note.uses().into_iter();
        let found = found.filter_map(|id| self.patterns.find(id).ok());
        found.map(|pattern| pattern.note).collect()
    }

    /// Adds `text` dim after `rail` and `label` padded to 8 characters, or after `rail` alone with no
    /// label.
    pub(super) fn labelled(
        &self,
        lines: &mut Vec<Row>,
        rail: &Styled,
        label: &'static str,
        text: &str,
    ) {
        lines.push(Row::Note {
            rail: rail.clone(),
            label,
            text: text.to_string(),
            tone: DIM,
        });
    }
}
