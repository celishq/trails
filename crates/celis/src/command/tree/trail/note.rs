//! The notes under a row: its text, proof, change, time, reason, and the steps it follows.

use crate::command::tree::row::Row;
use crate::command::tree::Tree;
use crate::terminal::canvas::{Styled, DIM};
use celis_trails::records::id::short;
use celis_trails::{Step, Used};

impl Tree<'_> {
    /// Adds under `note` the lines of its text, its expect labelled, its proof, change and with `grade`
    /// its grade when done, or its reason when dropped, after `rail`.
    pub(in crate::command::tree) fn closing(
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

    /// Adds under `note` a `follows` line for each step it follows, a `pattern` line for each pattern it
    /// uses, then a `followed` line when steps follow it, after `rail`.
    pub(in crate::command::tree) fn followed(&self, note: Step, rail: &Styled, out: &mut Vec<Row>) {
        for source in self.reuse.sources(note) {
            let line = format!("{} {}", short(source.id()), source.title());
            self.labelled(out, rail, "follows", &line);
        }
        for used in note.uses() {
            let pattern = match used {
                Used::Rule(id) => self.rules.get(id).filter(|(_, rule)| rule.is_pattern()),
                Used::Follow(_) => None,
            };
            if let Some((record, rule)) = pattern {
                let line = format!("{} {}", short(&record.id), rule.content);
                self.labelled(out, rail, "pattern", &line);
            }
        }
        let count = self.reuse.followers(note);
        if count > 0 {
            self.labelled(out, rail, "followed", &count.to_string());
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
