//! `celis prompt`: what a session's hook writes on the trail it works on.

use super::tree::today;
use super::{Args, Command};
use celis_trails::{Failure, Project, Trails};
use std::io::{Read, Write};
use std::path::Path;

impl Command {
    /// Writes a hook's prompt as a steer on the oldest working step of the session's open trail, else on
    /// the trail, signed now; prints its id, and names that step on `err`, then how a new ask joins a
    /// trail. With no open trail, writes nothing, and names the agent that brings past work on `err`, once
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
        if session.is_empty() || text.is_empty() || notified || !self.held()? {
            return Ok(());
        }
        let trails = self.trails()?;
        let steps = trails.steps();
        let Some(trail) = steps.session(&project.id, session) else {
            return self.recall(&trails, err, project, &asked);
        };
        let (record, working) = trails.prompt(&steps, trail, text)?;
        self.id(out, args, &record)?;
        if let Some((on, step)) = working.and_then(|n| Some((n, steps.step(n)?))) {
            let line = format!(
                "celis: you work on {} \"{}\"; when the next work does not fit it, add a step under it \
                 first, or replace it with step add --replaces.",
                celis_trails::id::short(&on.id),
                step.title()
            );
            super::put(err, &line)?;
        }
        super::put(err, &joins(Some(&trail.id)))
    }

    /// Names the agent that brings what `past` prints, when it prints anything and `asked` does not exist;
    /// then writes `asked`. Else, with a trail of today, says how a new ask joins it.
    fn recall(
        &self,
        trails: &Trails,
        err: &mut dyn Write,
        project: &Project,
        asked: &Path,
    ) -> Result<(), Failure> {
        let mut listed = Vec::new();
        if !asked.exists() {
            self.list(trails, &Args::default(), &mut listed, project)?;
        }
        if listed.is_empty() {
            return match today(&trails.steps(), trails.now).is_empty() {
                true => Ok(()),
                false => super::put(err, &joins(None)),
            };
        }
        super::put(
            err,
            "celis: a new ask in this session; before any other tool, run the celis:past agent in the \
             foreground with the ask and the places it may touch, and wait for its answer; it brings this \
             project's active trails and decisions, and the past work like the ask.",
        )?;
        if let Some(folder) = asked.parent() {
            std::fs::create_dir_all(folder).map_err(anyhow::Error::from)?;
        }
        std::fs::write(asked, "").map_err(anyhow::Error::from)?;
        Ok(())
    }
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
