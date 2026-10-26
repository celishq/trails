//! `celis patterns`: the board of the person's patterns.

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_trails::records::id::short;
use celis_trails::{Example, Pattern, Trails};
use std::io::Write;

impl Command {
    /// Prints each live pattern in scope, the rules first, newest first within each, with its examples,
    /// as lines or with `--markdown` as one page. Writes no record.
    pub(crate) fn patterns(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        if !self.held()? {
            return crate::command::put(out, NONE);
        }
        let trails = self.trails()?;
        let project = self.project().anchor.project;
        let mut patterns = trails.patterns().live(&project);
        patterns.sort_by_key(|pattern| !pattern.record.binds);
        if patterns.is_empty() {
            return crate::command::put(out, NONE);
        }
        if args.markdown {
            let mut page = vec!["# Patterns".to_string()];
            page.extend(
                patterns
                    .iter()
                    .flat_map(|pattern| markdown(pattern, &trails, &project)),
            );
            return crate::command::put(out, &page.join("\n"));
        }
        for pattern in &patterns {
            crate::command::put(out, &crate::text::line::pattern(pattern, &trails))?;
        }
        Ok(())
    }
}

/// The section of `pattern`: its first line as a heading, its id, scope and `rule` when it binds, the rest
/// of its text, then a line for each smaller pattern in scope in `project` and each example, a file as a
/// link to its blob.
fn markdown(pattern: &Pattern, trails: &Trails, project: &str) -> Vec<String> {
    let (note, pattern) = (pattern.note, &pattern.record);
    let mut text = pattern.content.lines();
    let mut head = vec![pattern.scope.text()];
    head.extend(pattern.binds.then(|| "rule".to_string()));
    let mut lines = vec![
        String::new(),
        format!("## {}", text.next().unwrap_or("")),
        String::new(),
        format!("`{}` · {}", short(&note.id), head.join(" · ")),
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
