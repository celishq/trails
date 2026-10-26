//! The prompt hook: what a session's prompt writes on the trail it works on.

use crate::command::read::tree::today;
use crate::command::Command;
use crate::failure::Failure;
use celis_store::{Link, Project};
use celis_trails::TrailStore;
use hook::Prompt;
use std::io::Write;
use std::path::{Path, PathBuf};

mod hook;

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

    /// Writes `prompt` on its session's open trail, signed now, else keeps it for the session's next step;
    /// then, unless it is a task, gives on `err` one line: how a new ask recalls and joins a trail.
    fn ask(&self, err: &mut dyn Write, project: &Project, prompt: Prompt) -> Result<(), Failure> {
        let offered = self.offered(&prompt.session)?;
        if !self.held()? {
            self.keep(&prompt)?;
            if !prompt.task {
                crate::command::put(err, &note(false, None, false))?;
                self.offer(err, project, &offered)?;
            }
            return Ok(());
        }
        let trails = self.trails()?;
        let steps = trails.steps();
        let trail = steps.session(&project.anchor.project, &prompt.session);
        match trail {
            Some(trail) => {
                trails.prompt(trail, prompt.text.clone())?;
            }
            None => self.keep(&prompt)?,
        }
        if prompt.task {
            return Ok(());
        }
        let id = trail.map(|t| celis_store::records::id::short(t.id()));
        let line = note(!trails.is_empty(), id.as_deref(), !today(&steps).is_empty());
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

/// The line on a prompt: a new ask takes the patterns that bear on it when the store holds a record,
/// then joins the session's trail `trail`, else a trail of `today`, else a new one; and how done work
/// is fixed.
fn note(recall: bool, trail: Option<&str>, today: bool) -> String {
    let join = match (trail, today) {
        (Some(id), _) => {
            format!("add it under the session's trail {id} when it fits, else with --new")
        }
        (None, true) => {
            "add it --under a related trail of today that celis tree shows, else with --new"
                .to_string()
        }
        (None, false) => "add it with --new".to_string(),
    };
    let ask = match recall {
        true => format!(
            "take the patterns that bear on it, from the index you got at the session start or from celis \
             patterns <file>... again, before you change a file, then {join}, with --recalled <the id \
             of each>"
        ),
        false => join,
    };
    let fix = match trail {
        Some(id) => format!(
            "; when it changes done work, add it with step add --replaces <the id of that step, as celis \
             tree {id} shows it>"
        ),
        None => String::new(),
    };
    format!("celis: when this prompt is a new ask, {ask}{fix}.")
}
