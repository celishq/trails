//! `celis patterns`: the board of the person's patterns.

use crate::command::{Args, Command};
use crate::failure::Failure;
use std::io::Write;

impl Command {
    /// Prints each live pattern in scope, newest first, with its examples. Writes no record.
    pub(crate) fn patterns(&self, _args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        if !self.held()? {
            return super::put(out, NONE);
        }
        let trails = self.trails()?;
        let live = trails.rules().live(&self.project().anchor.project);
        let patterns: Vec<_> = live.iter().filter(|(_, rule)| rule.is_pattern()).collect();
        if patterns.is_empty() {
            return super::put(out, NONE);
        }
        for (note, rule) in patterns {
            super::put(out, &crate::text::line::rule(note, rule, &trails))?;
        }
        Ok(())
    }
}

/// What `patterns` prints when the store holds none.
const NONE: &str = "no pattern yet";
