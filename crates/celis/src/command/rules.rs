//! `celis rules`: the rules one call matches.

mod given;
mod hook;

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_nostr::Note;
use celis_trails::{Call, Fit, Place, Project, Rule, State, Trails};
pub(super) use given::Given;
use hook::Hook;
use serde_json::json;
use std::io::Write;
use std::path::Path;

/// The most rules one call gives, but with `--all`.
const CAP: usize = 5;

impl Command {
    /// Prints the live rules in scope that the files and command of `args` match, else the call of the
    /// hook on stdin: those not yet given in the session, by `rank`, at most 5 but with `--all`, then
    /// how many more match. Logs each rule it gives in the session, with its working step. Writes no
    /// record.
    pub(crate) fn rules(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let files = &args.words[1..];
        let line = args.text.as_deref().filter(|t| !t.trim().is_empty());
        let (project, calls, session) = if files.is_empty() && line.is_none() {
            let hook = Hook::read()?;
            let folder = hook.folder.clone().unwrap_or_else(|| self.folder.clone());
            let project = Project::at(&folder);
            (project.clone(), hook.calls(&project, &folder), hook.session)
        } else {
            let project = self.project();
            let mut calls: Vec<Call> = files
                .iter()
                .map(|f| read(&project, &self.folder, f))
                .collect();
            calls.extend(line.map(|l| command(&project, &self.folder, l)));
            (project, calls, None)
        };
        let session = args.session.clone().or(session).or_else(|| self.session());
        if calls.is_empty() || !self.held()? {
            return Ok(());
        }
        let trails = self.trails()?;
        let given = Given::new(&self.home()?);
        let seen = session.as_deref().map(|s| given.of(s)).unwrap_or_default();
        let misses = given.misses();
        let live = trails.rules().live(&project.anchor.project).into_iter();
        let mut found: Vec<(Fit, &Note, Rule)> = live
            .filter(|(note, _)| !seen.contains(&note.id))
            .filter_map(|(note, rule)| {
                Some((calls.iter().filter_map(|c| rule.fit(c)).min()?, note, rule))
            })
            .collect();
        found.sort_by_key(|(fit, note, rule)| rank(rule, misses.contains(&note.id), *fit));
        let shown = if args.all {
            found.len()
        } else {
            found.len().min(CAP)
        };
        for (_, note, rule) in &found[..shown] {
            let object =
                json!({"id": note.id, "scope": rule.scope.text(), "content": rule.content});
            let line = crate::text::line::rule(note, rule);
            super::put(out, &if args.json { object.to_string() } else { line })?;
        }
        if let Some(session) = &session {
            let ids: Vec<&str> = found[..shown]
                .iter()
                .map(|(_, n, _)| n.id.as_str())
                .collect();
            let step = working(&trails, &project, session);
            given.give(session, &ids, step.as_deref(), super::now())?;
        }
        let more = found.len() - shown;
        if more == 0 {
            return Ok(());
        }
        let line = match session {
            Some(_) => format!(
                "celis: {more} more rules match this call; run it again for the next {CAP}, or add --all."
            ),
            None => format!("celis: {more} more rules match this call; add --all to see them."),
        };
        super::show(out, args.json, "more", json!(more), &line)
    }
}

/// Where a rule that fits a call by `fit` ranks, first first: made by a correction, then named by a miss
/// when `missed`, then the rest; within each, by `fit`.
fn rank(rule: &Rule, missed: bool, fit: Fit) -> (u8, Fit) {
    let evidence = match (rule.replaces.is_some(), missed) {
        (true, _) => 0,
        (false, true) => 1,
        (false, false) => 2,
    };
    (evidence, fit)
}

/// The id of the oldest working step of the open trail of `session` in `project`.
fn working(trails: &Trails, project: &Project, session: &str) -> Option<String> {
    let steps = trails.steps();
    let trail = steps.session(&project.anchor.project, session)?;
    let mut tree = trail.tree().into_iter().skip(1);
    tree.find(|s| s.state() == State::Working)
        .map(|s| s.id().to_string())
}

/// A read of the file `file`, given in `folder`, with its text on disk.
fn read(project: &Project, folder: &Path, file: &str) -> Call {
    Call::File {
        path: Place::resolve(file, &project.folder, folder),
        text: std::fs::read_to_string(folder.join(file)).ok(),
    }
}

/// The shell command `line`, run in `folder`, with the files it names.
fn command(project: &Project, folder: &Path, line: &str) -> Call {
    let named = Call::named(line).into_iter();
    Call::Command {
        line: line.to_string(),
        files: named
            .map(|w| Place::resolve(w, &project.folder, folder))
            .collect(),
    }
}

/// A read of each file that `git status --porcelain` gives in `project`.
fn changed(project: &Project) -> Vec<Call> {
    let status = std::process::Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=all"])
        .current_dir(&project.folder)
        .output();
    let Ok(status) = status else {
        return vec![];
    };
    let text = String::from_utf8_lossy(&status.stdout);
    let paths = text.lines().filter_map(|l| l.get(3..));
    let paths = paths.map(|p| p.rsplit(" -> ").next().unwrap_or(p).trim_matches('"'));
    paths.map(|p| read(project, &project.folder, p)).collect()
}
