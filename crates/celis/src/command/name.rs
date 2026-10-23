//! `celis name`: give a trail a name, or list the named trails.

use crate::command::{Args, Command};
use crate::failure::Failure;
use std::io::Write;

impl Command {
    /// Names the trail `args` names with the word after its id, and prints the id of the name record;
    /// with no trail, prints the named trails.
    pub(crate) fn name(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        if args.words.len() == 1 {
            return self.names(args, out);
        }
        let id = args.id("name")?;
        let name = args
            .words
            .get(2)
            .ok_or_else(|| Failure::usage("name needs a name after the trail id"))?;
        let written = self.trails()?.name(id, name)?;
        self.print(written, args, out, &mut std::io::sink())
    }
}
