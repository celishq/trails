//! `celis patterns`: the board of the person's patterns.

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_nostr::Note;
use celis_trails::records::id::short;
use celis_trails::{Example, Pattern, Trails};
use std::io::Write;

impl Command {
    /// Prints each live pattern in scope, the rules first, newest first within each, with its examples,
    /// as lines or with `--markdown` as one page. Writes no record.
    pub(crate) fn patterns(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        if !self.held()? {
            return super::put(out, NONE);
        }
        let trails = self.trails()?;
        let live = trails.patterns().live(&self.project().anchor.project);
        let mut patterns: Vec<_> = live.iter().collect();
        patterns.sort_by_key(|(_, pattern)| !pattern.binds);
        if patterns.is_empty() {
            return super::put(out, NONE);
        }
        if args.markdown {
            let mut page = vec!["# Patterns".to_string()];
            page.extend(
                patterns
                    .iter()
                    .flat_map(|(note, pattern)| markdown(note, pattern, &trails)),
            );
            return super::put(out, &page.join("\n"));
        }
        for (note, pattern) in patterns {
            super::put(out, &crate::text::line::pattern(note, pattern, &trails))?;
        }
        Ok(())
    }
}

/// The section of the pattern `note` holds: its first line as a heading, its id, scope and `rule` when
/// it binds, the rest of its text, then a line for each example, a file as a link to its blob.
fn markdown(note: &Note, pattern: &Pattern, trails: &Trails) -> Vec<String> {
    let mut text = pattern.content.lines();
    let mut head = vec![pattern.scope.short()];
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
