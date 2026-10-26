//! `celis patterns`: the index of the person's patterns.

use crate::command::record::files;
use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_patterns::{Example, Pattern, PatternStore};
use celis_store::records::id::short;
use celis_store::Store;
use celis_trails::TrailStore;
use std::collections::HashSet;
use std::io::Write;

impl Command {
    /// Prints the index of the live patterns in scope; with `--markdown`, each whole as one page, newest
    /// first. Writes no record.
    pub(crate) fn patterns(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        if !args.markdown {
            let index = self.index(&args.words[1..])?;
            return crate::command::put(out, index.as_deref().unwrap_or(NONE));
        }
        if !self.held()? {
            return crate::command::put(out, NONE);
        }
        let trails = self.trails()?;
        let project = self.project().anchor.project;
        let patterns = trails.patterns().live(&project);
        if patterns.is_empty() {
            return crate::command::put(out, NONE);
        }
        let mut page = vec!["# Patterns".to_string()];
        page.extend(
            patterns
                .iter()
                .flat_map(|pattern| markdown(pattern, &trails, &project)),
        );
        crate::command::put(out, &page.join("\n"))
    }

    /// The index of the live patterns in scope, oldest first: each one's id, scope and When, with its
    /// paths, each of `files` that they match, and its larger patterns; none when there is none.
    pub(crate) fn index(&self, files: &[String]) -> Result<Option<String>, Failure> {
        if !self.held()? {
            return Ok(None);
        }
        let trails = self.trails()?;
        let here = self.project();
        let patterns = trails.patterns().live(&here.anchor.project);
        if patterns.is_empty() {
            return Ok(None);
        }
        let every = trails.patterns();
        let held: HashSet<&str> = every.every().iter().map(|p| p.id()).collect();
        let files: Vec<(&str, Vec<String>)> = files
            .iter()
            .map(|f| (f.as_str(), files::paths(&here, &self.folder, f)))
            .collect();
        let lines: Vec<String> = patterns
            .iter()
            .rev()
            .map(|pattern| entry(pattern, &held, &files))
            .collect();
        Ok(Some(lines.join("\n")))
    }
}

/// The lines of `pattern` in the index: `<id8> <scope> <When>`, then each of its paths, each of the
/// `files` whose paths it matches, and each larger pattern of the `held` ids.
fn entry(pattern: &Pattern, held: &HashSet<&str>, files: &[(&str, Vec<String>)]) -> String {
    let record = &pattern.record;
    let head = format!(
        "{} {} {}",
        short(pattern.id()),
        record.scope.text(),
        record.when()
    );
    let paths = record.paths.iter().map(|p| format!("  path   {p}"));
    let matched = files.iter().filter(|(_, paths)| record.matches(paths));
    let matched = matched.map(|(file, _)| format!("  match  {file}"));
    let within = record.within.iter().filter(|w| held.contains(&**w as &str));
    let within = within.map(|w| format!("  within {}", short(w)));
    let lines = std::iter::once(head)
        .chain(paths)
        .chain(matched)
        .chain(within);
    lines.collect::<Vec<_>>().join("\n")
}

/// The section of `pattern`: its first line as a heading, its id and scope, the rest of its text, then a line for each smaller pattern in scope in `project` and each example, a file as a
/// link to its blob.
fn markdown(pattern: &Pattern, trails: &Store, project: &str) -> Vec<String> {
    let (note, pattern) = (pattern.note, &pattern.record);
    let mut text = pattern.content.lines();
    let mut lines = vec![
        String::new(),
        format!("## {}", text.next().unwrap_or("")),
        String::new(),
        format!("`{}` · {}", short(&note.id), pattern.scope.text()),
        String::new(),
    ];
    let rest: Vec<String> = text.map(|l| format!("{l}  ")).collect();
    if !rest.is_empty() {
        lines.extend(rest);
        lines.push(String::new());
    }
    let steps = trails.steps();
    for smaller in trails.patterns().smaller(project, &note.id) {
        let first = smaller.record.content.lines().next().unwrap_or("");
        lines.push(format!("- next: `{}` {first}", short(smaller.id())));
    }
    for (side, example) in pattern.examples.sides() {
        match example {
            Example::File(file) => {
                let at = crate::text::line::blob(file, trails);
                lines.push(format!("- {side}: [{}]({at})", file.place));
            }
            Example::Url(url) => lines.push(format!("- {side}: <{url}>")),
            Example::Step(id) => {
                let Ok(step) = steps.find(id) else { continue };
                let done = step.done();
                let proof = done.as_ref().and_then(|d| d.proof.lines().next());
                let head = format!("`{}` {}", short(step.id()), step.title());
                lines.push(format!("- {side}: {head} · {}", proof.unwrap_or("")));
                let files = done.iter().flat_map(|d| d.files.iter());
                lines.extend(files.map(|f| {
                    let at = crate::text::line::blob(f, trails);
                    format!("  - [{}]({at})", f.place)
                }));
            }
        }
    }
    lines
}

/// What `patterns` prints when the store holds none.
const NONE: &str = "no pattern yet";
