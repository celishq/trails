//! `celis migrate`: the patterns of a retired form, and with `--yes` their rewrite in the current form.

pub(crate) mod old;

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_store::records::id::short;
use old::Old;
use serde_json::json;
use std::io::Write;

/// What `migrate` prints when no pattern of a retired form is left.
const NOTHING: &str = "Nothing to migrate.";

impl Command {
    /// Prints each live pattern of a retired form, or with `--yes` writes each
    /// again and prints the id of each record written (`spec/02-cli.md`, "Migrate").
    pub(crate) fn migrate(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let store = self.trails()?;
        let old = Old::every(store.records());
        if old.is_empty() && !args.json {
            return crate::command::put(out, NOTHING);
        }
        if !args.yes {
            for pattern in &old {
                let first = pattern.note.content.lines().next().unwrap_or("");
                let line = format!(
                    "old {} {} {first}",
                    short(&pattern.note.id),
                    pattern.form.name()
                );
                let value = json!({"id": pattern.note.id, "form": pattern.form.name(),
                    "text": pattern.note.content});
                crate::command::show(out, args.json, "old", value, &line)?;
            }
            return Ok(());
        }
        for pattern in &old {
            for note in pattern.write(&store)? {
                self.id(out, args, &note)?;
            }
        }
        Ok(())
    }

    /// How many live patterns of a retired form the store holds.
    pub(crate) fn unmigrated(&self) -> usize {
        self.trails()
            .map_or(0, |store| Old::every(store.records()).len())
    }
}
