//! `celis decide`: write a rule.

use crate::command::rules::Given;
use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_trails::{Place, Project, Rule, Scope, Trigger};
use std::io::Write;

impl Command {
    /// Writes the rule `args` states on `project`, in its scope, with its trigger and labels, signed by
    /// the account now, and prints its id, then on `err` each override it ends. In a session, logs a
    /// miss of the rule it replaces when the session was not given it.
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
                tool: args.tool.clone(),
            },
            situation: args.situation,
            promoted: args.promoted,
        };
        let written = self.trails()?.decide(rule, args.replaces.as_deref())?;
        let replaced = written.records.first().and_then(Rule::read);
        if let (Some(session), Some(old)) = (self.session(), replaced.and_then(|r| r.replaces)) {
            let given = Given::new(&self.home()?);
            if !given.of(&session).contains(&*old) {
                given.miss(&session, &old, super::now())?;
            }
        }
        self.print(written, args, out, err)
    }
}
