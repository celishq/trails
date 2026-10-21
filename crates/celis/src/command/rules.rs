//! `celis rules`: the rules of some files and a command, or the index of the rules.

mod index;

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_nostr::Note;
use celis_trails::{Call, Fit, Place, Project, Rule};
use index::Index;
use serde_json::json;
use std::io::Write;
use std::path::Path;

impl Command {
    /// Prints every live rule in scope that the files and command of `args` match, by `rank`.
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
        let mut found: Vec<(Fit, &Note, Rule)> = live
            .filter_map(|(note, rule)| {
                Some((calls.iter().filter_map(|c| rule.fit(c)).min()?, note, rule))
            })
            .collect();
        found.sort_by_key(|(fit, _, rule)| rank(rule, *fit));
        for (_, note, rule) in &found {
            let object =
                json!({"id": note.id, "scope": rule.scope.text(), "content": rule.content});
            let line = crate::text::line::rule(note, rule);
            super::put(out, &if args.json { object.to_string() } else { line })?;
        }
        Ok(())
    }
}

/// Where a rule that fits a call by `fit` ranks, first first: made by a correction, then the rest;
/// within each, by `fit`.
fn rank(rule: &Rule, fit: Fit) -> (bool, Fit) {
    (rule.replaces.is_none(), fit)
}

/// The files `git <args>` lists in `folder`; none when git fails.
fn git(folder: &Path, args: &[&str]) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(folder)
        .output();
    let text = output.map_or_else(
        |_| String::new(),
        |o| String::from_utf8_lossy(&o.stdout).into(),
    );
    text.split('\0')
        .filter(|f| !f.is_empty())
        .map(str::to_string)
        .collect()
}

/// The files that git shows changed or new in `folder`, each that exists, each once.
fn changed(folder: &Path) -> Vec<String> {
    let listed = git(folder, &["diff", "-z", "--name-only", "--relative", "HEAD"]);
    let listed = listed.into_iter().chain(git(
        folder,
        &["ls-files", "-z", "--others", "--exclude-standard"],
    ));
    let mut files: Vec<String> = Vec::new();
    for file in listed.filter(|f| folder.join(f).is_file()) {
        if !files.contains(&file) {
            files.push(file);
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
    let listed = git(
        folder,
        &[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            file,
        ],
    );
    listed.iter().map(|f| read(project, folder, f)).collect()
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
