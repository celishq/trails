//! `celis lesson`: the learn brief of an ask or a step, at any time.

use crate::command::record::close::Learn;
use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_trails::TrailStore;
use std::io::Write;

impl Command {
    /// Prints the learn brief of the step `args` names, open or closed. Writes no record.
    pub(crate) fn lesson(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let id = args.id("lesson")?;
        if !self.held()? {
            return Err(Failure::no_step(id));
        }
        let trails = self.trails()?;
        let steps = trails.steps();
        let step = steps.find(id)?;
        let own = step.done().and_then(|d| d.changed);
        Learn::of(trails.records(), step, own)
            .lines()
            .iter()
            .try_for_each(|line| crate::command::put(out, line))
    }
}
