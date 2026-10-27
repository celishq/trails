//! `celis cell`: put patterns in cells.

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_patterns::{CellStore, Scope};
use std::io::Write;

impl Command {
    pub(crate) fn cell(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        match args.words.get(1).map(String::as_str).unwrap_or("") {
            "add" => self.cell_add(args, out),
            "on" => self.cell_turn(args, out, true),
            "off" => self.cell_turn(args, out, false),
            "export" => self.cell_export(args, out),
            "import" => self.cell_import(args, out),
            "drop" => self.cell_drop(args, out),
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

    /// Removes the cell of another account that `args` names from the store, and prints how many records
    /// went.
    fn cell_drop(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let name = args.id("cell drop")?;
        let gone = self.trails()?.leave(name)?;
        let line = format!(
            "dropped {name} · {}",
            crate::text::line::plural(gone, "record")
        );
        crate::command::put(out, &line)
    }

    /// Turns the cell `args` names on or off in the working folder's project, or in every project with
    /// `--scope me`, and prints the mask's id.
    fn cell_turn(&self, args: &Args, out: &mut dyn Write, on: bool) -> Result<(), Failure> {
        let command = if on { "cell on" } else { "cell off" };
        let name = args.id(command)?;
        let project = self.project();
        let anchor = match args.scope.as_deref().map(|s| (s, Scope::parse(s))) {
            None => Some(&project.anchor),
            Some((_, Some(Scope::Me))) => None,
            Some((scope, _)) => {
                return Err(Failure::usage(format!(
                    "{command} takes --scope me, not {scope}"
                )))
            }
        };
        let note = self.trails()?.turn(name, on, anchor)?;
        self.id(out, args, &note)
    }
}
