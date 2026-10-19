//! `celis stats`: what later work reused.

use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::progress::{Flags, Grade, Reuse};
use crate::text::line::plural;
use celis_trails::records::id::short;
use celis_trails::{Step, Trail};
use serde_json::json;
use std::io::{IsTerminal, Write};

/// What `stats` prints when nothing was reused.
const NOTHING: &str = "Nothing was reused yet.";

impl Command {
    /// Prints the step `args` names with the steps it follows and the steps that follow it, or with no id
    /// each followed step, each used decision and the total; on a terminal with no `--json`, draws them
    /// as cards. Writes no record.
    pub(crate) fn stats(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let id = args.words.get(1);
        if args.demo && id.is_some() {
            return Err(Failure::usage("stats takes an id or --demo, not both"));
        }
        if args.demo {
            return self.demo(args, out);
        }
        if std::io::stdout().is_terminal() && !args.json {
            return self.board(args, out);
        }
        if !self.held()? {
            return match id {
                Some(id) => Err(Failure::no_step(id)),
                None if args.json => Ok(()),
                None => super::put(out, NOTHING),
            };
        }
        let trails = self.trails()?;
        let steps = trails.steps();
        let reuse = Reuse::new(&steps);
        match id {
            Some(id) => chain(&reuse, Trail::new(steps.find(id)?).work(), args.json, out),
            None => {
                let decisions = trails.decisions();
                let used = reuse.used(&decisions);
                if reuse.followed().is_empty() && used.is_empty() {
                    return match args.json {
                        true => Ok(()),
                        false => super::put(out, NOTHING),
                    };
                }
                for &step in reuse.followed() {
                    let count = reuse.followers(step);
                    let revised = Flags::of(step).revised;
                    let value = json!({"id": step.id(), "project": step.record().anchor.name,
                        "content": step.title(), "followed": count, "revised": revised});
                    let mut line = format!("{} · followed {count}", head(step));
                    if revised {
                        line.push_str(" · revised");
                    }
                    super::show(out, args.json, "followed", value, &line)?;
                }
                let mut uses = 0;
                for (note, decision, count) in &used {
                    uses += count;
                    let value = json!({"id": note.id, "content": decision.content, "used": count});
                    let line = format!(
                        "rule {} {} · used {count}",
                        short(&note.id),
                        decision.brief()
                    );
                    super::show(out, args.json, "used", value, &line)?;
                }
                let follows = reuse.total();
                let value = json!({"follows": follows, "uses": uses});
                let line = format!(
                    "total {} · {}",
                    plural(follows, "follow"),
                    plural(uses, "use")
                );
                super::show(out, args.json, "total", value, &line)
            }
        }
    }
}

/// Prints `step` with its grade, then each step it follows, then each step that follows it, each at any
/// depth, two spaces further in a level.
fn chain(reuse: &Reuse, step: Step, json: bool, out: &mut dyn Write) -> Result<(), Failure> {
    let value = json!({"id": step.id(), "project": step.record().anchor.name,
        "content": step.title(), "grade": Grade::of(step), "depth": 0});
    let line = format!("{}{}", head(step), took(step));
    super::show(out, json, "step", value, &line)?;
    let sources = reuse.up(step).into_iter().map(|l| ("follows", l.source, l));
    let followers = reuse
        .down(step)
        .into_iter()
        .map(|l| ("followed", l.step, l));
    for (kind, at, link) in sources.chain(followers) {
        let value = json!({"id": at.id(), "project": at.record().anchor.name, "content": at.title(),
            "grade": Grade::of(at), "depth": link.depth});
        let indent = "  ".repeat(link.depth);
        let line = format!("{indent}{kind} {}{}", head(at), took(at));
        super::show(out, json, kind, value, &line)?;
    }
    Ok(())
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

mod board;
pub(crate) mod sheet;
mod table;
