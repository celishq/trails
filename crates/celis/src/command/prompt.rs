//! The prompt hook: what a session's prompt writes on the trail it works on.

use super::tree::today;
use crate::command::Command;
use crate::failure::Failure;
use celis_trails::{Link, Project, State};
use hook::Prompt;
use std::io::Write;
use std::path::Path;

mod hook;

impl Command {
    /// Acts on the hook `json` holds: a prompt goes on its session's open trail, with notes on `err`
    /// about the work.
    pub(super) fn act_on(
        &self,
        json: &serde_json::Value,
        err: &mut dyn Write,
    ) -> Result<(), Failure> {
        let project = self.project();
        match Prompt::of(json) {
            Some(prompt) => self.ask(err, &project, prompt),
            None => Ok(()),
        }
    }

    /// Writes `prompt` on its session's open trail, signed now; then names on `err` how a new ask
    /// searches its rules and patterns, the oldest working step, every done step, and how a new ask
    /// joins a trail.
    fn ask(&self, err: &mut dyn Write, project: &Project, prompt: Prompt) -> Result<(), Failure> {
        let asked = self.offered(&prompt.session)?;
        if !self.held()? {
            if !prompt.task {
                super::put(err, &fresh())?;
            }
            return self.offer(err, project, &asked);
        }
        let trails = self.trails()?;
        if !prompt.task && !trails.is_empty() {
            super::put(err, RECALL)?;
        }
        let steps = trails.steps();
        let Some(trail) = steps.session(&project.anchor.project, &prompt.session) else {
            if !prompt.task {
                let line = match today(&steps).is_empty() {
                    true => fresh(),
                    false => joins(None),
                };
                super::put(err, &line)?;
                self.offer(err, project, &asked)?;
            }
            return Ok(());
        };
        trails.prompt(trail, prompt.text)?;
        let working = trail.tree().into_iter().skip(1);
        if let Some(on) = working.into_iter().find(|s| s.state() == State::Working) {
            let line = format!(
                "celis: you work on {} \"{}\"; when the next work does not fit it, add a step under it \
                 first, or replace it with step add --replaces. To hand work to another agent, add a \
                 step for it, and put in its task: celis step <id>: run celis step start <id> first and \
                 celis step done <id> last.",
                celis_trails::records::id::short(on.id()),
                on.title()
            );
            super::put(err, &line)?;
        }
        self.offer(err, project, &asked)?;
        let done: Vec<_> = trail
            .tree()
            .into_iter()
            .skip(1)
            .filter(|s| s.state() == State::Done)
            .collect();
        if !done.is_empty() {
            super::put(
                err,
                "celis: when this prompt changes done work, add it with step add --replaces <its id>, \
                 not as a new ask; the done steps:",
            )?;
            for step in done {
                let line = format!(
                    "  {} \"{}\"",
                    celis_trails::records::id::short(step.id()),
                    step.title()
                );
                super::put(err, &line)?;
            }
        }
        super::put(err, &joins(Some(trail.id())))
    }

    /// The file that marks that `session` was asked to link the project.
    fn offered(&self, session: &str) -> Result<std::path::PathBuf, Failure> {
        let name = format!("{}.link", session.replace(['/', '.'], "_"));
        Ok(self.home()?.join("offered").join(name))
    }

    /// Asks, once in the session `offered` names, to link `project` to its `.celis/trails` when that
    /// folder exists and is not linked.
    fn offer(&self, err: &mut dyn Write, project: &Project, offered: &Path) -> Result<(), Failure> {
        let link = Link::new(project, Link::FOLDER);
        if offered.exists() || !link.folder.is_dir() || link.is_on(&self.home()?) {
            return Ok(());
        }
        super::put(
            err,
            "celis: this project keeps its trails in .celis/trails/, and this machine does not link \
             them; ask the person whether to link this project to celis, and if so, to run celis \
             link in it themselves. Never run celis link yourself, and never write in a linked folder.",
        )?;
        mark(offered)
    }
}

/// Writes the empty file `path`, and its folder.
fn mark(path: &Path) -> Result<(), Failure> {
    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder).map_err(anyhow::Error::from)?;
    }
    std::fs::write(path, "").map_err(anyhow::Error::from)?;
    Ok(())
}

/// How a new ask joins the session's open trail `open`, else a related trail of today, else a new one.
fn joins(open: Option<&str>) -> String {
    match open {
        Some(id) => format!(
            "celis: when this prompt is a new ask, add it under the session's trail {}, when it fits; \
             else run celis tree and add it --under a related trail of today; else add it --new. {RULES}",
            celis_trails::records::id::short(id)
        ),
        None => format!(
            "celis: when this prompt is a new ask, run celis tree and add it --under a related \
             trail of today; else start a new trail. {RULES}"
        ),
    }
}

/// How a new ask starts a trail when there is none to join.
fn fresh() -> String {
    format!("celis: when this prompt is a new ask, add it with --new. {RULES}")
}

/// What a new ask's line asks of the agent before it changes a file.
const RECALL: &str = "celis: when this prompt is a new ask, run celis search [<each file it will \
                      touch>]... -- \"<the ask>\" before you change a file, and keep each rule and \
                      pattern it prints; a prompt that goes on with the same work needs none.";

/// The sentence of a new ask's line that asks to write the rules the prompt states.
const RULES: &str = "When this prompt states how the work must be done (review feedback, a \
                     convention, a correction), write each point as a rule with celis decide before you \
                     change a file; give it --scope me when it holds for that kind of work in any project.";
