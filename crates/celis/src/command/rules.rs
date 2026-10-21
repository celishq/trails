//! `celis rules`: the rules of some files and a command.

mod given;

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_nostr::Note;
use celis_trails::{Call, Fit, Place, Project, Rule, State, Trails};
pub(super) use given::Given;
use serde_json::json;
use std::io::Write;
use std::path::Path;

impl Command {
    /// Prints every live rule in scope that the files and command of `args` match, by `rank`, and logs
    /// each in the session, with its working step. Writes no record.
    pub(crate) fn rules(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let files = &args.words[1..];
        let line = args.text.as_deref().filter(|t| !t.trim().is_empty());
        if files.is_empty() && line.is_none() {
            return Err(Failure::usage("rules needs a file or a command"));
        }
        if !self.held()? {
            return Ok(());
        }
        let project = self.project();
        let mut calls: Vec<Call> = files
            .iter()
            .map(|f| read(&project, &self.folder, f))
            .collect();
        calls.extend(line.map(|l| command(&project, &self.folder, l)));
        let trails = self.trails()?;
        let given = Given::new(&self.home()?);
        let misses = given.misses();
        let live = trails.rules().live(&project.anchor.project).into_iter();
        let mut found: Vec<(Fit, &Note, Rule)> = live
            .filter_map(|(note, rule)| {
                Some((calls.iter().filter_map(|c| rule.fit(c)).min()?, note, rule))
            })
            .collect();
        found.sort_by_key(|(fit, note, rule)| rank(rule, misses.contains(&note.id), *fit));
        for (_, note, rule) in &found {
            let object =
                json!({"id": note.id, "scope": rule.scope.text(), "content": rule.content});
            let line = crate::text::line::rule(note, rule);
            super::put(out, &if args.json { object.to_string() } else { line })?;
        }
        if let Some(session) = args.session.clone().or_else(|| self.session()) {
            let ids: Vec<&str> = found.iter().map(|(_, n, _)| n.id.as_str()).collect();
            let step = working(&trails, &project, &session);
            given.give(&session, &ids, step.as_deref(), super::now())?;
        }
        Ok(())
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
