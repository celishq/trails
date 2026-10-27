//! `celis stats`: the use of the patterns, or a step's grade.

use super::candidate::Candidate;
use crate::command::record::close::End;
use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::progress::{Grade, Model};
use crate::text::clock::date;
use crate::text::line::plural;
use celis_patterns::PatternStore;
use celis_store::records::id::short;
use celis_store::Store;
use celis_trails::{Step, Trail, TrailStore};
use lifecycle::Lifecycle;
use serde_json::json;
use std::io::{IsTerminal, Write};
use value::Value;

/// What `stats` prints when the store holds no record.
const NOTHING: &str = "Nothing was reused yet.";

/// The seconds of the last days that the total counts.
const WEEK: u64 = 7 * 86_400;

impl Command {
    /// Prints the step `args` names with its grade, or with no id each live pattern with its use, and the
    /// total; on a terminal with no `--json`, draws them as cards, or as the board with `--next`.
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
            Some(id) => step(&trails, Trail::new(steps.find(id)?).work(), args.json, out),
            None => uses(&trails, &self.project().anchor.project, args.json, out),
        }
    }
}

/// Prints each live pattern in scope in `project` with its use, oldest first, then the total, the
/// lifecycle and the value (CIP-02, "Stats").
fn uses(trails: &Store, project: &str, json: bool, out: &mut dyn Write) -> Result<(), Failure> {
    let steps = trails.steps();
    let live = Candidate::every(trails, project);
    for candidate in &live {
        let (pattern, chain) = (candidate.pattern(), candidate.chain());
        let id = pattern.id();
        let (uses, misled) = (steps.uses(id), steps.misled(id).len());
        let used = uses.first().map(|s| date(s.note().created_at));
        let examples = pattern.record.examples.sides().count();
        let tidy = candidate.reasons();
        let (name, when) = (pattern.record.name(), pattern.record.when());
        let value = json!({"id": id, "uses": uses.len(), "misled": misled, "replaces": chain,
            "used": used, "examples": examples, "tidy": tidy, "name": name, "when": when});
        let line = format!(
            "{} uses:{} misled:{misled} replaces:{chain} used:{} examples:{examples} tidy:{} {}",
            short(id),
            uses.len(),
            used.as_deref().unwrap_or("-"),
            match tidy.is_empty() {
                true => "-".to_string(),
                false => tidy.join(","),
            },
            pattern.record.title()
        );
        crate::command::show(out, json, "pattern", value, &line)?;
    }
    let since = steps.now().saturating_sub(WEEK);
    let patterns = trails.patterns();
    let scoped = patterns.view(project).scoped().into_iter();
    let written: Vec<_> = scoped.filter(|p| p.note.created_at >= since).collect();
    let replaced = written
        .iter()
        .filter(|p| p.record.replaces.is_some())
        .count();
    let unled = |s: &Step| s.trail().tree().iter().all(|t| t.uses().is_empty());
    let gaps = steps.all().into_iter().filter(|s| {
        s.record().anchor.project == project
            && s.note().created_at >= since
            && s.fixes()
            && unled(s)
    });
    let gaps = gaps.count();
    let value = json!({"index": live.len(), "written": written.len(), "replaced": replaced,
        "gaps": gaps});
    let line = format!(
        "total {} in the index · last 7 days: {} written, {replaced} replaced, {}",
        plural(live.len(), "pattern"),
        written.len(),
        plural(gaps, "gap")
    );
    crate::command::show(out, json, "total", value, &line)?;
    let (spans, value) = Model::of(trails, |m| {
        (Lifecycle::of(m, Some(project)), Value::of(m, Some(project)))
    });
    for span in spans {
        crate::command::show(out, json, "lifecycle", span.json(), &span.line())?;
    }
    crate::command::show(out, json, "value", value.json(), &value.line())
}

/// Prints `step` with its grade, then with no `json`, of a done ask, its total from `trails`.
fn step(trails: &Store, step: Step, json: bool, out: &mut dyn Write) -> Result<(), Failure> {
    let value = json!({"id": step.id(), "project": step.record().anchor.name,
        "content": step.title(), "grade": Grade::of(step), "depth": 0});
    let line = format!("{}{}", head(step), took(step));
    crate::command::show(out, json, "step", value, &line)?;
    if json || !step.is_trail() {
        return Ok(());
    }
    let Some(done) = step.done() else {
        return Ok(());
    };
    let end = End::of(trails.records(), step, &done);
    crate::command::put(out, &end.total())
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
mod lifecycle;
mod measures;
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
mod value;

#[cfg(test)]
mod tests;
