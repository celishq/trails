//! The prompt hook: what a session's prompt writes on the trail it works on.

use crate::command::Command;
use crate::failure::Failure;
use celis_store::{Link, Project, Secret};
use celis_trails::{State, Step, TrailStore};
use hook::Prompt;
use std::io::Write;
use std::path::{Path, PathBuf};

mod hook;

/// The line on a prompt that holds a secret.
const SECRET: &str = "celis: the prompt holds a secret, so celis keeps no record of it; never write the \
                      secret in a celis command, and name it by where it lives, such as API_KEY in .env.";

impl Command {
    /// Acts on the hook `json` holds: a prompt goes on its session's open trail, with notes on `err`
    /// about the work.
    pub(crate) fn act_on(
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

    /// Writes `prompt` on its session's open trail, signed now, else keeps it for the session's next step,
    /// unless it holds a secret;
    /// then, unless it is a task, gives on `err` one line: how a step of the current ask and a new ask
    /// are added, and when a pattern of the index is read.
    fn ask(&self, err: &mut dyn Write, project: &Project, prompt: Prompt) -> Result<(), Failure> {
        let offered = self.offered(&prompt.session)?;
        let kept = Secret::find(&prompt.text).is_none();
        if !kept && !prompt.task {
            crate::command::put(err, SECRET)?;
        }
        if !self.held()? {
            if kept {
                self.keep(&prompt)?;
            }
            if !prompt.task {
                crate::command::put(err, &note(false, None))?;
                self.offer(err, project, &offered)?;
            }
            return Ok(());
        }
        let trails = self.trails()?;
        let steps = trails.steps();
        let trail = steps.session(&project.anchor.project, &prompt.session);
        match trail {
            Some(trail) if kept => {
                trails.prompt(trail, prompt.text.clone())?;
            }
            None if kept => self.keep(&prompt)?,
            _ => {}
        }
        if prompt.task {
            return Ok(());
        }
        let current = steps.current(&project.anchor.project, &prompt.session);
        if let Some(ask) = current.filter(|t| waits(*t)) {
            let id = celis_store::records::id::short(ask.id());
            crate::command::put(err, &format!("celis: no step under the ask {id} is open; when its work is done, close it with celis step done {id}."))?;
        }
        let id = current.map(|t| celis_store::records::id::short(t.id()));
        let line = note(!trails.is_empty(), id.as_deref());
        crate::command::put(err, &line)?;
        self.offer(err, project, &offered)
    }

    /// The file that keeps the newest prompt of `session` that came with no open trail.
    fn kept(&self, session: &str) -> Result<PathBuf, Failure> {
        let name = session.replace(['/', '.'], "_");
        Ok(self.home()?.join("prompts").join(name))
    }

    /// Keeps `prompt` for its session's next step.
    fn keep(&self, prompt: &Prompt) -> Result<(), Failure> {
        let path = self.kept(&prompt.session)?;
        if let Some(folder) = path.parent() {
            std::fs::create_dir_all(folder).map_err(anyhow::Error::from)?;
        }
        std::fs::write(path, &prompt.text).map_err(anyhow::Error::from)?;
        Ok(())
    }

    /// Writes the prompt kept for `session` on the trail of `step`, and forgets it.
    pub(crate) fn place_kept(&self, session: &str, step: &str) -> Result<(), Failure> {
        let path = self.kept(session)?;
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Ok(());
        };
        let trails = self.trails()?;
        let steps = trails.steps();
        trails.prompt(steps.find(step)?.trail(), text)?;
        std::fs::remove_file(path).map_err(anyhow::Error::from)?;
        Ok(())
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
        crate::command::put(
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

/// Whether the ask `ask` is open, with steps under it and none of them open.
fn waits(ask: Step) -> bool {
    let steps = ask.children();
    ask.state() == State::Open && !steps.is_empty() && steps.iter().all(|s| s.closed().is_some())
}

/// The line on a prompt: a prompt that goes on with the current ask `current` adds steps under it, and a
/// new ask is a new step; how done work is fixed; and, when the store holds a record, when a pattern is
/// read.
fn note(held: bool, current: Option<&str>) -> String {
    let ask = match current {
        Some(id) => format!(
            "when this prompt goes on with the ask {id}, add its steps --under {id}; when it is a new ask, \
             add it with celis step add --new"
        ),
        None => "when this prompt is a new ask, add it with celis step add".to_string(),
    };
    let load = match held {
        true => " When a choice of the work matches a When of the index, read that pattern with celis \
                 patterns <id> before you choose, and name it with --uses <id> on the step of that choice.",
        false => "",
    };
    format!(
        "celis: {ask}; when it changes done work, add it with step add --replaces <the id of that step, as \
         celis tree shows it>.{load}"
    )
}
