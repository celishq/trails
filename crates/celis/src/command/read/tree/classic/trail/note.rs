//! The notes under a row: its text, proof, change, time, reason, and the patterns it uses.

use crate::command::read::tree::classic::row::Row;
use crate::command::read::tree::classic::Tree;
use crate::terminal::canvas::{Styled, DIM};
use celis_store::records::id::short;
use celis_trails::Step;

impl Tree<'_> {
    /// Adds under `note` the lines of its text, its expect labelled, its proof, change and with `grade`
    /// its grade when done, or its reason when dropped, after `rail`.
    pub(in crate::command::read::tree::classic) fn closing(
        &self,
        note: Step,
        rail: &Styled,
        grade: bool,
        out: &mut Vec<Row>,
    ) {
        for line in note.record().detail() {
            match line.strip_prefix("expect:") {
                Some(expect) => self.labelled(out, rail, "expect", expect.trim_start()),
                None => self.labelled(out, rail, "", line),
            }
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

    /// Adds under `note` a `pattern` line for each pattern it uses, after `rail`.
    pub(in crate::command::read::tree::classic) fn used(
        &self,
        note: Step,
        rail: &Styled,
        out: &mut Vec<Row>,
    ) {
        for id in note.uses() {
            if let Ok(pattern) = self.patterns.find(None, id) {
                let line = format!("{} {}", short(pattern.id()), pattern.record.content);
                self.labelled(out, rail, "pattern", &line);
            }
        }
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
