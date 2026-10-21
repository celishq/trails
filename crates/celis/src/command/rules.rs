//! `celis rules`: the rules of some files and a command, or the index of the rules.

mod given;
mod index;

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_nostr::Note;
use celis_trails::records::id::short;
use celis_trails::{Call, Fit, Place, Project, Rule, State, Trails};
pub(super) use given::Given;
use index::Index;
use serde_json::json;
use std::io::Write;
use std::path::Path;

impl Command {
    /// Prints every live rule in scope that the files and command of `args` match, by `rank`, and logs
    /// each in the session, with its working step; a rule given before in the session prints as its id.
    /// With no file and no command, prints the index of the live rules in scope. Writes no record.
    pub(crate) fn rules(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let mut files = args.words[1..].to_vec();
        if args.diff {
            files.extend(changed(&self.folder));
        }
        let line = args.text.as_deref().filter(|t| !t.trim().is_empty());
        if !self.held()? {
            return Ok(());
        }
        let project = self.project();
        let trails = self.trails()?;
        let live = trails.rules().live(&project.anchor.project).into_iter();
        if files.is_empty() && line.is_none() && !args.diff {
            let index = Index::of(live.map(|(_, rule)| rule.content));
            return super::put(out, &index.text());
        }
        let mut calls: Vec<Call> = files
            .iter()
            .flat_map(|f| reads(&project, &self.folder, f))
            .collect();
        calls.extend(line.map(|l| command(&project, &self.folder, l)));
        let given = Given::new(&self.home()?);
        let misses = given.misses();
        let mut found: Vec<(Fit, &Note, Rule)> = live
            .filter_map(|(note, rule)| {
                Some((calls.iter().filter_map(|c| rule.fit(c)).min()?, note, rule))
            })
            .collect();
        found.sort_by_key(|(fit, note, rule)| rank(rule, misses.contains(&note.id), *fit));
        let live = trails.rules().live(&project.anchor.project).into_iter();
        let waiting = live
            .filter(|(_, rule)| calls.iter().any(|c| rule.waits(c)))
            .count();
        let session = args.session.clone().or_else(|| self.session());
        let before = session.as_deref().map(|s| given.of(s)).unwrap_or_default();
        let (old, new): (Vec<_>, Vec<_>) = found
            .iter()
            .partition(|(_, note, _)| before.contains(&note.id));
        for (_, note, rule) in &new {
            let object =
                json!({"id": note.id, "scope": rule.scope.text(), "content": rule.content});
            let line = crate::text::line::rule(note, rule);
            super::put(out, &if args.json { object.to_string() } else { line })?;
        }
        if !old.is_empty() {
            let ids = old.iter().map(|(_, n, _)| n.id.as_str());
            let text = match args.json {
                true => json!({"given": ids.collect::<Vec<_>>()}).to_string(),
                false => format!(
                    "given before: {}",
                    ids.map(short).collect::<Vec<_>>().join(" ")
                ),
            };
            super::put(out, &text)?;
        }
        if waiting > 0 {
            let missing: Vec<&str> = files
                .iter()
                .map(String::as_str)
                .filter(|f| !self.folder.join(f).exists())
                .collect();
            let text = match args.json {
                true => json!({ "waiting": waiting }).to_string(),
                false => format!(
                    "{waiting} more rules may match the text of {}; run celis rules again after you write them",
                    missing.join(" ")
                ),
            };
            super::put(out, &text)?;
        }
        if let Some(session) = session {
            let ids: Vec<&str> = new.iter().map(|(_, n, _)| n.id.as_str()).collect();
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

/// The files that git shows changed or new in `folder`, each that exists.
fn changed(folder: &Path) -> Vec<String> {
    let listed = |args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(folder)
            .output();
        output.map_or_else(
            |_| String::new(),
            |o| String::from_utf8_lossy(&o.stdout).into(),
        )
    };
    let text = listed(&["diff", "--name-only", "--relative", "HEAD"])
        + &listed(&["ls-files", "--others", "--exclude-standard"]);
    let mut files: Vec<String> = Vec::new();
    for file in text.lines().filter(|f| folder.join(f).is_file()) {
        if !files.iter().any(|f| f == file) {
            files.push(file.to_string());
        }
    }
    files
}

/// A read of the file `file`, given in `folder`, else of each file git gives under it when it is a
/// folder.
fn reads(project: &Project, folder: &Path, file: &str) -> Vec<Call> {
    if !folder.join(file).is_dir() {
        return vec![read(project, folder, file)];
    }
    let listed = std::process::Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            file,
        ])
        .current_dir(folder)
        .output();
    let text = listed.map_or_else(
        |_| String::new(),
        |o| String::from_utf8_lossy(&o.stdout).into(),
    );
    let files = text.split('\0').filter(|f| !f.is_empty());
    files.map(|f| read(project, folder, f)).collect()
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
