//! `celis extract`: the learn brief of an ask or a step, at any time.

use crate::command::record::close::Learn;
use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_trails::TrailStore;
use std::io::Write;

impl Command {
    /// Prints the learn brief of the step `args` names, open or closed, as one JSON object with
    /// `--json`. Writes no record.
    pub(crate) fn extract(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let id = args.id("extract")?;
        if !self.held()? {
            return Err(Failure::no_step(id));
        }
        let trails = self.trails()?;
        let steps = trails.steps();
        let step = steps.find(id)?;
        let own = step.done().and_then(|d| d.changed);
        let learn = Learn::of(trails.records(), step, own);
        if args.json {
            return crate::command::put(out, &learn.json().to_string());
        }
        learn
            .lines()
            .iter()
            .try_for_each(|line| crate::command::put(out, line))
    }
}
