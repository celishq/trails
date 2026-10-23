//! `celis decide`: write a rule.

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_trails::{Place, Project, Rule, Scope};
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
        let trails = self.trails()?;
        let steps = trails.steps();
        let step = |id: &Option<String>| -> Result<Option<String>, Failure> {
            match id {
                Some(id) => Ok(Some(steps.find(id)?.id().to_string())),
                None => Ok(None),
            }
        };
        let (good, bad) = (step(&args.good)?, step(&args.bad)?);
        let rule = Rule {
            anchor: project.anchor.clone(),
            scope,
            replaces: None,
            content,
            paths: args
                .paths
                .iter()
                .map(|p| Place::resolve(p, &project.folder, &self.folder))
                .collect(),
            situation: args.situation,
            promoted: args.promoted,
            good,
            bad,
        };
        let written = trails.decide(rule, args.replaces.as_deref())?;
        self.print(written, args, out, err)
    }
}
