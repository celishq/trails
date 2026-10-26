//! `celis stats`: what later work reused.

use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::progress::{Grade, Reuse};
use crate::text::line::plural;
use celis_patterns::PatternStore;
use celis_store::records::id::short;
use celis_trails::{Step, Trail, TrailStore};
use serde_json::json;
use std::io::{IsTerminal, Write};

/// What `stats` prints when nothing was reused.
const NOTHING: &str = "Nothing was reused yet.";

impl Command {
    /// Prints the step `args` names with its grade, or with no id each used pattern, each pattern with
    /// examples or steps it misled, and the total; on a terminal with no `--json`, draws them as cards, or as the
    /// board with `--next`. Writes no record.
    pub(crate) fn stats(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let id = args.words.get(1);
        if args.demo && id.is_some() {
            return Err(Failure::usage("stats takes an id or --demo, not both"));
        }
        if args.demo {
            return self.demo(args, out);
        }
        if std::io::stdout().is_terminal() && !args.json {
            return match args.next {
                true => self.board(args, out),
                false => self.cards(args, out),
            };
        }
        if !self.held()? {
            return match id {
                Some(id) => Err(Failure::no_step(id)),
                None if args.json => Ok(()),
                None => crate::command::put(out, NOTHING),
            };
        }
        let trails = self.trails()?;
        let steps = trails.steps();
        match id {
            Some(id) => step(Trail::new(steps.find(id)?).work(), args.json, out),
            None => {
                let reuse = Reuse::new(&steps);
                let patterns = trails.patterns();
                let used = reuse.used(&patterns);
                let live = patterns.live(&self.project().anchor.project);
                let patterns: Vec<_> = live
                    .iter()
                    .map(|p| {
                        let id = &p.note.id;
                        (p, steps.misled(id).len())
                    })
                    .filter(|(p, misled)| !p.record.examples.is_empty() || *misled > 0)
                    .collect();
                if used.is_empty() && patterns.is_empty() {
                    return match args.json {
                        true => Ok(()),
                        false => crate::command::put(out, NOTHING),
                    };
                }
                let mut uses = 0;
                for (pattern, count) in &used {
                    let (note, pattern) = (pattern.note, &pattern.record);
                    uses += count;
                    let value = json!({"id": note.id, "content": pattern.content, "used": count});
                    let line = format!(
                        "pattern {} {} · used {count}",
                        short(&note.id),
                        pattern.brief()
                    );
                    crate::command::show(out, args.json, "used", value, &line)?;
                }
                for (pattern, misled) in patterns {
                    let (note, pattern) = (pattern.note, &pattern.record);
                    let examples = pattern.examples.sides().count();
                    let value = json!({"id": note.id, "content": pattern.content,
                        "examples": examples, "misled": misled});
                    let mut line = format!(
                        "pattern {} {} · {}",
                        short(&note.id),
                        pattern.brief(),
                        plural(examples, "example")
                    );
                    if misled > 0 {
                        line.push_str(&format!(" · misled {misled}"));
                    }
                    crate::command::show(out, args.json, "pattern", value, &line)?;
                }
                let value = json!({"uses": uses});
                let line = format!("total {}", plural(uses, "use"));
                crate::command::show(out, args.json, "total", value, &line)
            }
        }
    }
}

/// Prints `step` with its grade.
fn step(step: Step, json: bool, out: &mut dyn Write) -> Result<(), Failure> {
    let value = json!({"id": step.id(), "project": step.record().anchor.name,
        "content": step.title(), "grade": Grade::of(step), "depth": 0});
    let line = format!("{}{}", head(step), took(step));
    crate::command::show(out, json, "step", value, &line)
}

/// `<id, first 8> [<project name>] <title>` of `step`.
fn head(step: Step) -> String {
    let name = &step.record().anchor.name;
    format!("{} [{name}] {}", short(step.id()), step.title())
}

/// ` · <grade>` of a done `step`, else nothing.
fn took(step: Step) -> String {
    crate::text::line::grade(step).map_or(String::new(), |g| format!(" · {g}"))
}

mod allocation;
mod board;
mod classic;
mod daily;
mod head;
mod kind;
mod learning;
mod moment;
mod month;
mod outcomes;
mod output;
mod panel;
mod project;
mod quality;
mod section;
mod sheet;
mod total;

#[cfg(test)]
mod tests;
