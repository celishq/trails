//! `celis rules`: the rules of some files, or the index of the rules.

pub(super) mod index;

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_nostr::Note;
use celis_trails::{Example, Fit, Place, Project, Rule};
use index::Index;
use serde_json::json;
use std::io::Write;
use std::path::Path;

impl Command {
    /// Prints every live rule in scope that the files of `args` match, by `rank`, each with the proof and
    /// its good and bad examples.
    /// With no file, prints the index of the live rules in scope. Writes no record.
    pub(crate) fn rules(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let mut files = args.words[1..].to_vec();
        if args.diff {
            files.extend(changed(&self.folder));
        }
        if !self.held()? {
            return Ok(());
        }
        let project = self.project();
        let trails = self.trails()?;
        let live = trails.rules().live(&project.anchor.project).into_iter();
        if files.is_empty() && !args.diff {
            let index = Index::of(live.map(|(_, rule)| rule.content));
            return super::put(out, &index.text());
        }
        let paths: Vec<String> = files
            .iter()
            .flat_map(|f| paths(&project, &self.folder, f))
            .collect();
        let mut found: Vec<(Fit, &Note, Rule)> = live
            .filter_map(|(note, rule)| Some((rule.fit(&paths)?, note, rule)))
            .collect();
        found.sort_by_key(|(fit, _, rule)| rank(rule, *fit));
        for (_, note, rule) in &found {
            let mut object =
                json!({"id": note.id, "scope": rule.scope.text(), "content": rule.content});
            for (side, list) in [("good", &rule.examples.good), ("bad", &rule.examples.bad)] {
                if !list.is_empty() {
                    object[side] = json!(list.iter().map(value).collect::<Vec<_>>());
                }
            }
            let text = if args.json {
                object.to_string()
            } else {
                crate::text::line::rule(note, rule, &trails)
            };
            super::put(out, &text)?;
        }
        Ok(())
    }
}

/// What the JSON of a rule gives for `example`: the id of its step, the sha256 of its file, or its URL.
fn value(example: &Example) -> String {
    match example {
        Example::Step(id) => id.to_string(),
        Example::File(file) => file.sha256.clone(),
        Example::Url(url) => url.clone(),
    }
}

/// Where a rule that fits a file by `fit` ranks, first first: made by a correction, then the rest;
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
    let new = git(
        folder,
        &["ls-files", "-z", "--others", "--exclude-standard"],
    );
    let mut files: Vec<String> = Vec::new();
    for file in listed.into_iter().chain(new) {
        if folder.join(&file).is_file() && !files.contains(&file) {
            files.push(file);
        }
    }
    files
}

/// The path of the file `file`, given in `folder`, from the project's top folder; else of each file
/// git gives under it when it is a folder.
pub(super) fn paths(project: &Project, folder: &Path, file: &str) -> Vec<String> {
    let files = match folder.join(file).is_dir() {
        true => git(
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
        ),
        false => vec![file.to_string()],
    };
    let files = files.iter();
    files
        .map(|f| Place::resolve(f, &project.folder, folder))
        .collect()
}
