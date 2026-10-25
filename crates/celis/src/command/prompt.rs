//! The prompt hook: what a session's prompt writes on the trail it works on.

use super::tree::today;
use crate::command::Command;
use crate::failure::Failure;
use celis_trails::{Link, Project};
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

    /// Writes `prompt` on its session's open trail, signed now; then, unless it is a task, gives on `err`
    /// one line: how a new ask searches and joins a trail, and how a correction or a reaction is kept.
    fn ask(&self, err: &mut dyn Write, project: &Project, prompt: Prompt) -> Result<(), Failure> {
        let offered = self.offered(&prompt.session)?;
        if !self.held()? {
            if !prompt.task {
                super::put(err, &note(false, None, false))?;
                self.offer(err, project, &offered)?;
            }
            return Ok(());
        }
        let trails = self.trails()?;
        let steps = trails.steps();
        let trail = steps.session(&project.anchor.project, &prompt.session);
        if let Some(trail) = trail {
            trails.prompt(trail, prompt.text)?;
        }
        if prompt.task {
            return Ok(());
        }
        let id = trail.map(|t| celis_trails::records::id::short(t.id()));
        let line = note(!trails.is_empty(), id.as_deref(), !today(&steps).is_empty());
        super::put(err, &line)?;
        self.offer(err, project, &offered)
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

/// The line on a prompt: a new ask runs `search` when the store holds a record, then joins the
/// session's trail `trail`, else a trail of `today`, else a new one; how done work is fixed; and how a
/// correction or a reaction becomes a pattern.
fn note(search: bool, trail: Option<&str>, today: bool) -> String {
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
    let ask = match search {
        true => format!(
            "run celis search [<each file it will touch>]... -- \"<the ask>\" before you change a file, \
             then {join}"
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
    format!(
        "celis: when this prompt is a new ask, {ask}{fix}; when it says always or never, or states a \
         convention, write a rule with celis decide --binds before you change a file, with --scope me when \
         it holds in any project; when it reacts to a result, such as \"this one\" or \"too much\", write a \
         pattern with celis decide --good <what the person liked> --bad <what they did not>."
    )
}
