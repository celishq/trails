//! `celis prompt`: what a session's hook writes on the trail it works on.

use super::tree::today;
use super::{Args, Command};
use celis_trails::{Failure, Link, Project, State, Trails};
use std::io::{Read, Write};
use std::path::Path;

impl Command {
    /// Writes a hook's prompt on the session's open trail, signed now; prints its id, names the oldest
    /// working step on `err`, then how a new ask joins a trail. With no open trail, writes nothing, and names the agent that brings past work on `err`, once
    /// a session. For a harness message, does neither. A compaction's `SessionStart` lets the session name
    /// the agent again.
    pub(super) fn prompt(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        let hook = hook()?;
        let session = hook["conversation_id"]
            .as_str()
            .or(hook["session_id"].as_str())
            .unwrap_or("");
        let asked = self
            .home()?
            .join("asked")
            .join(session.replace(['/', '.'], "_"));
        if !session.is_empty() && hook["source"] == "compact" {
            let _ = std::fs::remove_file(&asked);
            return Ok(());
        }
        let text: String = hook["prompt"]
            .as_str()
            .unwrap_or("")
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .chars()
            .take(200)
            .collect();
        let notified = ["<task-notification>", "<agent-message"]
            .iter()
            .any(|t| text.starts_with(t));
        if session.is_empty() || text.is_empty() || notified {
            return Ok(());
        }
        if !self.held()? {
            return self.offer(err, project, &asked);
        }
        let trails = self.trails()?;
        let steps = trails.steps();
        let Some(trail) = steps.session(&project.id, session) else {
            if !self.recall(&trails, err, project, &asked)? {
                self.offer(err, project, &asked)?;
            }
            return Ok(());
        };
        let record = trails.prompt(trail, text)?;
        self.id(out, args, &record)?;
        let working = trail.tree().into_iter().skip(1);
        if let Some(on) = working.into_iter().find(|s| s.state() == State::Working) {
            let line = format!(
                "celis: you work on {} \"{}\"; when the next work does not fit it, add a step under it \
                 first, or replace it with step add --replaces.",
                celis_trails::id::short(on.id()),
                on.title()
            );
            super::put(err, &line)?;
        }
        self.offer(err, project, &asked)?;
        super::put(err, &joins(Some(trail.id())))
    }

    /// Names the agent that brings what `past` prints, when it prints anything and `asked` does not exist;
    /// then writes `asked`. Else, with a trail of today, says how a new ask joins it. Returns whether it
    /// named the agent.
    fn recall(
        &self,
        trails: &Trails,
        err: &mut dyn Write,
        project: &Project,
        asked: &Path,
    ) -> Result<bool, Failure> {
        let mut listed = Vec::new();
        if !asked.exists() {
            self.list(trails, &Args::default(), &mut listed, project)?;
        }
        if listed.is_empty() {
            if !today(&trails.steps(), trails.now).is_empty() {
                super::put(err, &joins(None))?;
            }
            return Ok(false);
        }
        super::put(
            err,
            "celis: a new ask in this session; before any other tool, run the celis:past agent in the \
             foreground with the ask and the places it may touch, and wait for its answer; it brings this \
             project's active trails and decisions, and the past work like the ask.",
        )?;
        mark(asked)?;
        Ok(true)
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
             else run celis tree and add it --under a related trail of today; else add it --new.",
            celis_trails::id::short(id)
        ),
        None => "celis: when this prompt is a new ask, run celis tree and add it --under a related \
                 trail of today; else start a new trail."
            .to_string(),
    }
}

/// The hook's JSON on stdin; a usage error when it is no JSON.
fn hook() -> Result<serde_json::Value, Failure> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(anyhow::Error::from)?;
    serde_json::from_str(&input).map_err(|_| Failure::usage("expected a hook's JSON on stdin"))
}
