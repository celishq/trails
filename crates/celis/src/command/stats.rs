//! `celis stats`: what later work reused, and what it saved.

use super::{Args, Command};
use celis_nostr::Note;
use celis_trails::id::short;
use celis_trails::trail::grade::saved;
use celis_trails::trail::reuse::{sum, Link};
use celis_trails::{Failure, Reuse, Steps};
use serde_json::json;
use std::io::Write;

/// What `stats` prints when nothing was reused.
const NOTHING: &str = "Nothing was reused yet.";

impl Command {
    /// Prints the step `args` names with the steps it follows and the steps that follow it, or with no id
    /// each followed step, each used decision and the total. Writes no record.
    pub(super) fn stats(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let id = args.words.get(1);
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
            Some(id) => {
                let note = steps.find(id)?;
                let note = steps.twin(note).unwrap_or(note);
                chain(&steps, &reuse, note, args.json, out)
            }
            None => {
                let decisions = trails.decisions();
                let followed = reuse.followed();
                let used = reuse.used(&decisions);
                if followed.is_empty() && used.is_empty() {
                    return match args.json {
                        true => Ok(()),
                        false => super::put(out, NOTHING),
                    };
                }
                for note in followed {
                    let count = reuse.down(note).len();
                    let save = reuse.saved(note);
                    let value = json!({"id": note.id, "project": project(&steps, note),
                        "content": steps.title(note), "followed": count, "saved": save,
                        "revised": !steps.revised(&note.id).is_empty()});
                    let mut line = format!("{} · followed {count}", head(&steps, note));
                    line.push_str(&saving(save));
                    if !steps.revised(&note.id).is_empty() {
                        line.push_str(" · revised");
                    }
                    super::show(out, args.json, "followed", value, &line)?;
                }
                let mut uses = 0;
                for (note, decision, count) in &used {
                    uses += count;
                    let value = json!({"id": note.id, "content": decision.content, "used": count});
                    let line = format!(
                        "decision {} {} · used {count}",
                        short(&note.id),
                        decision.brief()
                    );
                    super::show(out, args.json, "used", value, &line)?;
                }
                let links = reuse.links();
                let save = sum(links.iter().map(|l| reuse.saving(l)));
                let value = json!({"follows": links.len(), "saved": save, "uses": uses});
                let line = format!(
                    "total {}{} · {}",
                    plural(links.len(), "follow"),
                    saving(save),
                    plural(uses, "use")
                );
                super::show(out, args.json, "total", value, &line)
            }
        }
    }
}

/// Prints `note` with its grade, then each step it follows, then each step that follows it, each at any
/// depth, two spaces further in a level.
fn chain(
    steps: &Steps,
    reuse: &Reuse,
    note: &Note,
    json: bool,
    out: &mut dyn Write,
) -> Result<(), Failure> {
    let grade = |n: &Note| steps.grade(n);
    let value = json!({"id": note.id, "project": project(steps, note), "content": steps.title(note),
        "grade": grade(note).map(|g| g.value()), "depth": 0});
    let line = format!("{}{}", head(steps, note), took(steps, note));
    super::show(out, json, "step", value, &line)?;
    let sources = reuse
        .up(note)
        .into_iter()
        .map(|l| ("follows", l.source, l, None));
    let followers = reuse
        .down(note)
        .into_iter()
        .map(|l| ("followed", l.step, l, reuse.saving(&l)));
    for (kind, at, link, save) in sources.chain(followers) {
        let Link { depth, .. } = link;
        let value = json!({"id": at.id, "project": project(steps, at), "content": steps.title(at),
            "grade": grade(at).map(|g| g.value()), "saved": save, "depth": depth});
        let indent = "  ".repeat(depth);
        let line = format!(
            "{indent}{kind} {}{}{}",
            head(steps, at),
            took(steps, at),
            saving(save)
        );
        super::show(out, json, kind, value, &line)?;
    }
    Ok(())
}

/// `<id, first 8> [<project name>] <title>` of `note`.
fn head(steps: &Steps, note: &Note) -> String {
    let name = steps.step(note).map(|s| s.anchor.name).unwrap_or_default();
    format!("{} [{name}] {}", short(&note.id), steps.title(note))
}

/// The project name of `note`.
fn project(steps: &Steps, note: &Note) -> String {
    steps.step(note).map(|s| s.anchor.name).unwrap_or_default()
}

/// ` · <grade>` of a done `note`, else nothing.
fn took(steps: &Steps, note: &Note) -> String {
    steps
        .grade(note)
        .map_or(String::new(), |g| format!(" · {}", g.line()))
}

/// ` · saved <time>`, or nothing with no saving.
fn saving(save: Option<i64>) -> String {
    save.map_or(String::new(), |s| format!(" · saved {}", saved(s)))
}

/// `<n> <word>s`, or `1 <word>`.
fn plural(n: usize, word: &str) -> String {
    match n {
        1 => format!("1 {word}"),
        n => format!("{n} {word}s"),
    }
}
