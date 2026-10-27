//! `celis cells`: each cell, and whether it is on here.

use crate::command::Command;
use crate::failure::Failure;
use celis_nostr::Pubkey;
use celis_patterns::PatternStore;
use std::io::Write;

impl Command {
    /// Prints each cell: its name, its account, how many live patterns it holds, and whether it is on
    /// in the working folder's project.
    pub(crate) fn cells(&self, out: &mut dyn Write) -> Result<(), Failure> {
        if !self.held()? {
            return crate::command::put(out, NONE);
        }
        let trails = self.trails()?;
        let patterns = trails.patterns();
        let project = self.project().anchor.project;
        let mut lines = vec![];
        for cell in patterns.cells().all() {
            let live = patterns.every().iter().filter(|p| p.is_live());
            let held = live.filter(|p| patterns.holding(p).contains(&cell)).count();
            let npub = Pubkey::parse(&cell.author).map_err(Failure::from)?.npub();
            let state = match patterns.view(&project).is_on(cell) {
                true => "on",
                false => "off",
            };
            let count = crate::text::line::plural(held, "pattern");
            lines.push(format!("{} {} {count} {state}", cell.name, &npub[5..13]));
        }
        match lines.is_empty() {
            true => crate::command::put(out, NONE),
            false => crate::command::put(out, &lines.join("\n")),
        }
    }
}

/// What `cells` prints when there is no cell.
const NONE: &str = "no cell yet";
