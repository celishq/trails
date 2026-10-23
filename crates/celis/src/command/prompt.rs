//! `celis prompt`: what a session's hook writes on the trail it works on.

use super::tree::today;
use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_trails::{Link, Project, State, Trails};
use hook::{Hook, Prompt};
use std::io::Write;
use std::path::Path;

mod hook;

impl Command {
    /// Acts on a hook: a prompt goes on its session's open trail, with notes on `err` about the work;
    /// with no open trail, it names the past agent, once a session, unless the prompt is a task. A
    /// compaction lets the session name the past agent again.
    pub(crate) fn prompt(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        self.act(Hook::read()?, args, out, err, project)
    }

    /// Acts on the hook `json` holds, as `prompt` acts on the hook on stdin.
    pub(super) fn act_on(
        &self,
        json: &serde_json::Value,
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> Result<(), Failure> {
        self.act(Hook::of(json), &Args::default(), out, err, &self.project())
    }

    fn act(
        &self,
        hook: Hook,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        match hook {
            Hook::Compaction { session } => {
                let _ = std::fs::remove_file(self.asked(&session)?);
                Ok(())
            }
            Hook::Prompt(prompt) => self.ask(args, out, err, project, prompt),
            Hook::Nothing => Ok(()),
        }
    }

    /// Writes `prompt` on its session's open trail, signed now; prints its id, then names on `err` the
    /// oldest working step, every done step, and how a new ask joins a trail.
    fn ask(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
        project: &Project,
        prompt: Prompt,
    ) -> Result<(), Failure> {
        let asked = self.asked(&prompt.session)?;
        if !self.held()? {
            if !prompt.task {
                super::put(err, &fresh())?;
            }
            return self.offer(err, project, &asked);
        }
        let trails = self.trails()?;
        let steps = trails.steps();
        let Some(trail) = steps.session(&project.anchor.project, &prompt.session) else {
            if !prompt.task && !self.recall(&trails, err, &asked)? {
                self.offer(err, project, &asked)?;
            }
            return Ok(());
        };
        let record = trails.prompt(trail, prompt.text)?;
        self.id(out, args, &record)?;
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

    /// Names the past agent, when the store holds any record and `asked` does not exist; then writes
    /// `asked`. Else says how a new ask joins a trail of today, or starts one. Returns whether it named
    /// the agent.
    fn recall(&self, trails: &Trails, err: &mut dyn Write, asked: &Path) -> Result<bool, Failure> {
        if asked.exists() || trails.is_empty() {
            let line = if today(&trails.steps()).is_empty() {
                fresh()
            } else {
                joins(None)
            };
            super::put(err, &line)?;
            return Ok(false);
        }
        let line = format!(
            "celis: a new ask in this session; before any other tool, {PAST}; it brings the rules the \
             ask must keep and the steps of past asks like it; add the ask with --uses <id> for each rule \
             and past step the ask follows. {RULES}"
        );
        super::put(err, &line)?;
        mark(asked)?;
        Ok(true)
    }

    /// The file that marks that `session` named the past agent.
    fn asked(&self, session: &str) -> Result<std::path::PathBuf, Failure> {
        Ok(self
            .home()?
            .join("asked")
            .join(session.replace(['/', '.'], "_")))
    }

    /// Asks, once in the session `asked` names, to link `project` to its `.celis/trails` when that folder
    /// exists and is not linked.
    fn offer(&self, err: &mut dyn Write, project: &Project, asked: &Path) -> Result<(), Failure> {
        let link = Link::new(project, Link::FOLDER);
        let offered = asked.with_extension("link");
        if offered.exists() || !link.folder.is_dir() || link.is_on(&self.home()?) {
            return Ok(());
        }
        super::put(
            err,
            "celis: this project keeps its trails in .celis/trails/, and this machine does not link \
             them; ask the person whether to link this project to celis, and if so, to run celis \
             link in it themselves.",
        )?;
        mark(&offered)
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

/// What a new ask's line asks of the agent before any other tool.
pub(super) const PAST: &str =
    "run the celis:past agent in the foreground with the ask and the places it \
     may touch, never in the background, and wait for its answer before any other tool";

/// The sentence of a new ask's line that asks to write the rules the prompt states.
const RULES: &str = "When this prompt states how the work must be done (review feedback, a \
                     convention, a correction), write each point as a rule with celis decide before you \
                     change a file; give it --scope me when it holds for that kind of work in any project.";
