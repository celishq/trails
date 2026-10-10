//! `celis decide`: write a decision of yours.

use super::{Args, Command};
use crate::steps::ACTIVE;
use crate::{Decision, Decisions, Failure, Project, Scope, Step, Steps};
use std::io::Write;

impl Command {
    /// Writes the decision `args` states on `project`, in its scope, signed by the account at `now`, and
    /// prints its id.
    pub(super) fn decide(
        &self,
        args: &Args,
        out: &mut dyn Write,
        project: &Project,
        now: u64,
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
        let store = self.store()?;
        let records = store.records();
        let scope = match scope {
            Scope::Trail(id) => {
                let steps = Steps { records: &records };
                let note = steps.find(&id).map_err(|_| Failure::no_trail(&id))?;
                let active = steps.state(note) == ACTIVE
                    && Step::read(note)
                        .is_some_and(|s| s.under.is_none() && s.anchor.project == project.id);
                match active {
                    true => Scope::Trail(note.id.clone()),
                    false => return Err(Failure::no_trail(&id)),
                }
            }
            scope => scope,
        };
        let replaces = match &args.replaces {
            Some(id) => {
                let decisions = Decisions {
                    records: &records,
                    account: store.account().pubkey(),
                };
                Some(decisions.find(&project.id, id)?.id.clone())
            }
            None => None,
        };
        let decision = Decision {
            anchor: project.anchor(),
            scope,
            replaces,
            content,
        };
        let written = decision.record(store.account(), now)?;
        store.write(&written)?;
        self.id(out, args, &written)
    }
}
