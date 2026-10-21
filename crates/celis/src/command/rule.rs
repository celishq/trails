//! `celis decide`: write a rule.

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_trails::{Place, Project, Rule, Scope, Trigger};
use std::io::Write;

impl Command {
    /// Writes the rule `args` states on `project`, in its scope, with its trigger and labels, signed by
    /// the account now, and prints its id, then on `err` each override it ends.
    pub(crate) fn decide(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
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
        let rule = Rule {
            anchor: project.anchor.clone(),
            scope,
            replaces: None,
            content,
            trigger: Trigger {
                paths: args
                    .paths
                    .iter()
                    .map(|p| Place::resolve(p, &project.folder, &self.folder))
                    .collect(),
                content: args.content.clone(),
                command: args.command.clone(),
            },
            situation: args.situation,
            promoted: args.promoted,
        };
        let written = self.trails()?.decide(rule, args.replaces.as_deref())?;
        self.print(written, args, out, err)
    }
}
