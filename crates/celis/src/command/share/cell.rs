//! `celis cell`: put patterns in cells.

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_patterns::PatternStore;
use std::io::Write;

impl Command {
    pub(crate) fn cell(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        match args.words.get(1).map(String::as_str).unwrap_or("") {
            "add" => self.cell_add(args, out),
            "" => Err(Failure::usage("expected: celis cell <command>")),
            verb => Err(Failure::usage(format!("no command \"cell {verb}\""))),
        }
    }

    /// Puts the patterns `args` names in the cell it names, and prints the membership's id.
    fn cell_add(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let name = args.id("cell add")?;
        let note = self.trails()?.join(name, &args.words[3..])?;
        self.id(out, args, &note)
    }
}
