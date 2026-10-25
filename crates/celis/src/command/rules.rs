//! The live rules that some files match.

use crate::command::Command;
use celis_nostr::Note;
use celis_trails::{Fit, Place, Project, Rule, Trails};
use std::path::Path;

impl Command {
    /// The live rules in scope that `files` match, by `rank`.
    pub(super) fn matched<'a>(
        &self,
        trails: &'a Trails,
        files: &[String],
    ) -> Vec<(&'a Note, Rule)> {
        let project = self.project();
        let paths: Vec<String> = files
            .iter()
            .flat_map(|f| paths(&project, &self.folder, f))
            .collect();
        let live = trails.rules().live(&project.anchor.project).into_iter();
        let mut found: Vec<(Fit, &Note, Rule)> = live
            .filter_map(|(note, rule)| Some((rule.fit(&paths)?, note, rule)))
            .collect();
        found.sort_by_key(|(fit, _, rule)| rank(rule, *fit));
        found
            .into_iter()
            .map(|(_, note, rule)| (note, rule))
            .collect()
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
pub(super) fn changed(folder: &Path) -> Vec<String> {
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
