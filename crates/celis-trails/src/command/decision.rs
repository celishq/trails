//! `celis decide`: write a decision.

use super::{Args, Command};
use crate::{Decision, Failure, Project, Scope, State, Step};
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
        let snapshot = self.snapshot()?;
        let scope = match scope {
            Scope::Trail(id) => {
                let steps = snapshot.steps();
                let note = steps.find(&id).map_err(|_| Failure::no_trail(&id))?;
                let active = steps.state(note) == State::Active
                    && Step::read(note)
                        .is_some_and(|s| s.under.is_none() && s.anchor.project == project.id);
                match active {
                    true => Scope::Trail(note.id.clone()),
                    false => return Err(Failure::no_trail(&id)),
                }
            }
            Scope::At(place) => Scope::At(project.place(&place, &self.folder)),
            scope => scope,
        };
        let replaces = match &args.replaces {
            Some(id) => Some(snapshot.decisions().find(&project.id, id)?.id.clone()),
            None => None,
        };
        let decision = Decision {
            anchor: project.anchor(),
            scope,
            replaces,
            content,
        };
        let written = snapshot.sign(|account, now| decision.record(account, now))?;
        self.id(out, args, &written)
    }
}
