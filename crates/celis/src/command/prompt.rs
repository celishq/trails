//! `celis prompt`: what a session's hook writes on the trail it works on.

use super::{Args, Command};
use celis_trails::{Failure, Project, Trails};
use std::io::{Read, Write};

impl Command {
    /// Writes a hook's prompt as a steer on the oldest working step of the session's open trail, else on
    /// the trail, signed now; prints its id, and names that step on `err`. With no open trail, writes
    /// nothing, and names the agent that brings past work on `err`. For a harness message, does neither.
    pub(super) fn prompt(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        let hook = hook()?;
        let session = hook["session_id"].as_str().unwrap_or("");
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
            return self.recall(&trails, err, project);
        };
        let (record, working) = trails.prompt(&steps, trail, text)?;
        self.id(out, args, &record)?;
        let Some((on, step)) = working.and_then(|n| Some((n, steps.step(n)?))) else {
            return Ok(());
        };
        let line = format!(
            "celis: you work on {} \"{}\"; when the next work does not fit it, add a step under it first, \
             or replace it with step add --replaces.",
            celis_trails::id::short(&on.id),
            step.title()
        );
        super::put(err, &line)
    }

    /// Names the agent that brings what `past` prints, when it prints anything.
    fn recall(
        &self,
        trails: &Trails,
        err: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        let mut listed = Vec::new();
        self.list(trails, &Args::default(), &mut listed, project)?;
        if listed.is_empty() {
            return Ok(());
        }
        super::put(
            err,
            "celis: a new ask in this session; before any other tool, run the celis:past agent in the \
             foreground with the ask and the places it may touch, and wait for its answer; it brings this \
             project's active trails and decisions, and the past work like the ask.",
        )
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
