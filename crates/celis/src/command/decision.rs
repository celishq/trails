//! `celis decide`: write a decision.

use super::{Args, Command};
use celis_trails::{Decision, Failure, Project, Scope};
use std::io::Write;

impl Command {
    /// Writes the decision `args` states on `project`, in its scope, signed by the account now, and
    /// prints its id.
    pub(super) fn decide(
        &self,
        args: &Args,
        out: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        let content = args.required_text("decide needs its text after --")?;
        let scope = match &args.scope {
            Some(text) => Scope::parse(text).ok_or_else(|| {
                Failure::usage(format!(
                    "a scope is me, project, trail:<id> or at:<place>, not {text}"
                ))
            })?,
            None => Scope::Project,
        };
        let scope = match scope {
            Scope::At(place) => Scope::At(project.place(&place, &self.folder)),
            scope => scope,
        };
        let decision = Decision {
            anchor: project.anchor(),
            scope,
            replaces: args.replaces.clone(),
            content,
        };
        let written = self.trails()?.decide(decision)?;
        self.print(written, args, out, &mut std::io::sink())
    }
}
