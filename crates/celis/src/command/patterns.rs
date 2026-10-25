//! `celis patterns`: the board of the person's patterns.

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_nostr::Note;
use celis_trails::records::id::short;
use celis_trails::{Example, Rule, Trails};
use std::io::Write;

impl Command {
    /// Prints each live pattern in scope, newest first, with its examples, as lines or with `--markdown`
    /// as one page. Writes no record.
    pub(crate) fn patterns(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        if !self.held()? {
            return super::put(out, NONE);
        }
        let trails = self.trails()?;
        let live = trails.rules().live(&self.project().anchor.project);
        let patterns: Vec<_> = live.iter().filter(|(_, rule)| rule.is_pattern()).collect();
        if patterns.is_empty() {
            return super::put(out, NONE);
        }
        if args.markdown {
            let mut page = vec!["# Patterns".to_string()];
            page.extend(
                patterns
                    .iter()
                    .flat_map(|(note, rule)| markdown(note, rule, &trails)),
            );
            return super::put(out, &page.join("\n"));
        }
        for (note, rule) in patterns {
            super::put(out, &crate::text::line::rule(note, rule, &trails))?;
        }
        Ok(())
    }
}

/// The section of the pattern `note` holds: its text as a heading, its id and scope, then a line for each
/// example, a file as a link to its blob.
fn markdown(note: &Note, rule: &Rule, trails: &Trails) -> Vec<String> {
    let mut lines = vec![
        String::new(),
        format!("## {}", rule.content),
        String::new(),
        format!("`{}` · {}", short(&note.id), rule.scope.short()),
        String::new(),
    ];
    let steps = trails.steps();
    for (side, example) in rule.examples.sides() {
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
