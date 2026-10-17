//! `celis decide`: write a decision.

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_trails::{Decision, Place, Project, Scope};
use std::io::Write;

impl Command {
    /// Writes the decision `args` states on `project`, in its scope, signed by the account now, and
    /// prints its id.
    pub(crate) fn decide(
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
            Scope::At(place) => Scope::At(Place::resolve(&place, &project.folder, &self.folder)),
            scope => scope,
        };
        let decision = Decision {
            anchor: project.anchor(),
            scope,
            replaces: None,
            content,
        };
        let written = self.trails()?.decide(decision, args.replaces.as_deref())?;
        self.print(written, args, out, &mut std::io::sink())
    }
}
