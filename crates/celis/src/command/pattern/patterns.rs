//! `celis patterns`: the index of the person's patterns.

use crate::command::pattern::candidate::Candidate;
use crate::command::pattern::level::Level;
use crate::command::step::files;
use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::text::image;
use crate::text::line::plural;
use crate::text::markdown;
use celis_patterns::{Example, Pattern, PatternStore};
use celis_store::records::id::short;
use celis_store::Store;
use celis_store::{ProofFile, Refusal};
use celis_trails::TrailStore;
use std::collections::HashSet;
use std::io::{IsTerminal, Write};

impl Command {
    /// Prints the index of the live patterns in scope; with an id, that pattern whole; with `--markdown`,
    /// each whole as one page, newest first; with `--tidy`, those that need tidy and the index size.
    pub(crate) fn patterns(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        if args.tidy {
            if args.markdown || args.words.len() > 1 {
                return Err(Failure::usage("patterns takes --tidy alone"));
            }
            let (mut lines, size) = self.candidates()?;
            lines.push(format!("{} in the index", plural(size, "pattern")));
            return crate::command::put(out, &lines.join("\n"));
        }
        if let Some(name) = &args.cell {
            if args.markdown || args.words.len() > 1 {
                return Err(Failure::usage("patterns takes --cell alone"));
            }
            return crate::command::put(out, &self.cell_index(name)?);
        }
        if let [id] = &args.words[1..] {
            if is_id(id) && !self.folder.join(id).exists() {
                return self.pattern_whole(id, out);
            }
        }
        if !args.markdown {
            let index = self.index(&args.words[1..])?;
            return crate::command::put(out, index.as_deref().unwrap_or(NONE));
        }
        if !self.held()? {
            return crate::command::put(out, NONE);
        }
        let trails = self.trails()?;
        let project = self.project().anchor.project;
        let patterns = trails.patterns().view(&project).live();
        if patterns.is_empty() {
            return crate::command::put(out, NONE);
        }
        let steps = trails.steps();
        let level = Level::of(&steps);
        let mut page = vec!["# Patterns".to_string()];
        page.extend(
            patterns
                .iter()
                .flat_map(|pattern| markdown(pattern, &level, &trails, &project)),
        );
        crate::command::put(out, &page.join("\n"))
    }

    /// Prints the live pattern in scope whose id is or starts with `id` whole, as `--markdown` does,
    /// and recalls it on the session's newest open step.
    fn pattern_whole(&self, id: &str, out: &mut dyn Write) -> Result<(), Failure> {
        if !self.held()? {
            return Err(Refusal::NoPattern(id.to_string()).into());
        }
        let trails = self.trails()?;
        let project = self.project().anchor.project;
        let pattern = trails.patterns().view(&project).find(id)?;
        let steps = trails.steps();
        let open = self.session().and_then(|s| steps.open(&project, &s));
        if let Some(step) = open {
            trails.recall(step.id(), pattern.id())?;
        }
        let level = Level::of(&steps);
        let mut lines = markdown(&pattern, &level, &trails, &project);
        while lines.last().is_some_and(String::is_empty) {
            lines.pop();
        }
        crate::command::put(out, &lines[1..].join("\n"))?;
        let program = self.env.get("TERM_PROGRAM").map(String::as_str);
        if !image::drawn(std::io::stdout().is_terminal(), program) {
            return Ok(());
        }
        for file in pictures(&pattern, &trails) {
            let Some(bytes) = trails
                .blob(&file.sha256)
                .and_then(|p| std::fs::read(p).ok())
            else {
                continue;
            };
            crate::command::put(out, &image::inline(&file.place, &bytes))?;
        }
        Ok(())
    }

    /// The index of the live patterns of the cells named `name`, on or off, oldest first, as `index`
    /// gives them; refuses a name that no cell holds.
    fn cell_index(&self, name: &str) -> Result<String, Failure> {
        if !self.held()? {
            return Err(Refusal::NoCell(name.to_string()).into());
        }
        let trails = self.trails()?;
        let patterns = trails.patterns();
        if patterns.cells().named(name).is_empty() {
            return Err(Refusal::NoCell(name.to_string()).into());
        }
        let project = self.project().anchor.project;
        let every = patterns.every().iter().rev();
        let view = patterns.view(&project);
        let held = every.clone().filter(|p| view.shows(p));
        let held: HashSet<&str> = held.map(|p| p.id()).collect();
        let ours = every.filter(|p| p.is_live());
        let ours = ours.filter(|p| patterns.holding(p).iter().any(|c| c.name == name));
        let steps = trails.steps();
        let level = Level::of(&steps);
        let lines: Vec<String> = ours.map(|p| entry(p, &level, &held, &[])).collect();
        Ok(match lines.is_empty() {
            true => NONE.to_string(),
            false => lines.join("\n"),
        })
    }

    /// The line of each live pattern in scope that needs tidy, oldest first, and how many are live in
    /// scope.
    pub(crate) fn candidates(&self) -> Result<(Vec<String>, usize), Failure> {
        if !self.held()? {
            return Ok((Vec::new(), 0));
        }
        let trails = self.trails()?;
        let project = self.project().anchor.project;
        let size = trails.patterns().view(&project).live().len();
        let found = Candidate::all(&trails, &project);
        Ok((found.iter().map(Candidate::line).collect(), size))
    }

    /// The index of the live patterns in scope, oldest first: each one's id, scope and When, with its
    /// paths, each of `files` that they match, and its larger patterns; none when there is none.
    pub(crate) fn index(&self, files: &[String]) -> Result<Option<String>, Failure> {
        if !self.held()? {
            return Ok(None);
        }
        let trails = self.trails()?;
        let here = self.project();
        let patterns = trails.patterns().view(&here.anchor.project).live();
        if patterns.is_empty() {
            return Ok(None);
        }
        let every = trails.patterns();
        let view = every.view(&here.anchor.project);
        let held = every.every().iter().filter(|p| view.shows(p));
        let held: HashSet<&str> = held.map(|p| p.id()).collect();
        let files: Vec<(&str, Vec<String>)> = files
            .iter()
            .map(|f| (f.as_str(), files::paths(&here, &self.folder, f)))
            .collect();
        let steps = trails.steps();
        let level = Level::of(&steps);
        let lines: Vec<String> = patterns
            .iter()
            .rev()
            .map(|pattern| entry(pattern, &level, &held, &files))
            .collect();
        Ok(Some(lines.join("\n")))
    }
}

/// The lines of `pattern` in the index: `<id8> <scope> [<level>] <Name>`, then each of its paths, each
/// of the `files` whose paths it matches, and each larger pattern of the `held` ids.
fn entry(
    pattern: &Pattern,
    level: &Level,
    held: &HashSet<&str>,
    files: &[(&str, Vec<String>)],
) -> String {
    let record = &pattern.record;
    let mark = level.mark(pattern.id()).map(|m| format!(" {m}"));
    let head = format!(
        "{} {}{} {}",
        short(pattern.id()),
        record.scope.text(),
        mark.unwrap_or_default(),
        record.title()
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

/// The section of `pattern`: its Name, else its first line, as a heading, its id and scope, the rest of its text, then a line for each smaller pattern in scope in `project` and each example, a file as a
/// link to its blob.
fn markdown(pattern: &Pattern, level: &Level, trails: &Store, project: &str) -> Vec<String> {
    let (note, record) = (pattern.note, &pattern.record);
    let mark = level.mark(&note.id).map(|m| format!(" · {m}"));
    let meta = format!(
        "`{}` · {}{}",
        short(&note.id),
        record.scope.text(),
        mark.unwrap_or_default()
    );
    let mut tail = vec![];
    for smaller in trails.patterns().view(project).smaller(&note.id) {
        let first = smaller.record.content.lines().next().unwrap_or("");
        let title = smaller.record.name().unwrap_or(first);
        tail.push(format!("- next: `{}` {title}", short(smaller.id())));
    }
    let steps = trails.steps();
    for (side, example) in record.examples.sides() {
        if let Example::Step(id) = example {
            let Ok(step) = steps.find(id) else { continue };
            let done = step.done();
            let proof = done.as_ref().and_then(|d| d.proof.lines().next());
            let head = format!("`{}` {}", short(step.id()), step.title());
            tail.push(format!("- {side}: {head} · {}", proof.unwrap_or("")));
            let files = done.iter().flat_map(|d| d.files.iter());
            tail.extend(files.map(|f| {
                let at = crate::text::line::blob(f, trails);
                format!("  - {}", markdown::file_link(f, &at))
            }));
            continue;
        }
        let link = match example {
            Example::File(file) => crate::text::line::blob(file, trails),
            _ => String::new(),
        };
        tail.extend(markdown::example(side, example, &link));
    }
    markdown::section(record, Some(meta), tail)
}

/// Whether `word` reads as a record id: 4 to 64 hex characters.
fn is_id(word: &str) -> bool {
    (4..=64).contains(&word.len()) && word.chars().all(|c| c.is_ascii_hexdigit())
}

/// The image files of `pattern`: its file examples, and the files of its done steps.
fn pictures(pattern: &Pattern, trails: &Store) -> Vec<ProofFile> {
    let steps = trails.steps();
    let mut files = vec![];
    for (_, example) in pattern.record.examples.sides() {
        match example {
            Example::File(file) => files.push(file.clone()),
            Example::Step(id) => {
                let Ok(step) = steps.find(id) else { continue };
                let done = step.done();
                files.extend(done.iter().flat_map(|d| d.files.iter()).cloned());
            }
            Example::Url(_) => {}
        }
    }
    files.retain(|file| file.media.starts_with("image/"));
    files
}

/// What `patterns` prints when the store holds none.
const NONE: &str = "no pattern yet";
