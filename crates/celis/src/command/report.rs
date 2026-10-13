//! `celis report`: the trails that ids name, as Markdown for a pull request.

use super::tree::step_look;
use super::{Args, Command};
use celis_nostr::Note;
use celis_trails::id::short;
use celis_trails::trail::grade::took;
use celis_trails::{Failure, Status, Steps, Trails};
use serde_json::json;
use std::io::Write;

impl Command {
    /// Prints what `markdown` gives for the ids `args` names. Writes no record.
    pub(super) fn report(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let ids = &args.words[1..];
        if ids.is_empty() {
            return Err(Failure::usage("report needs an id"));
        }
        if !self.held()? {
            return Err(Failure::no_step(&ids[0]));
        }
        let text = markdown(&self.trails()?, ids)?;
        super::show(out, args.json, "report", json!(text), text.trim_end())
    }
}

/// The trail of the step each of `ids` names, once, in their order, as Markdown.
pub(super) fn markdown(trails: &Trails, ids: &[String]) -> Result<String, Failure> {
    let steps = trails.steps();
    let decisions = trails.decisions();
    let mut chosen: Vec<&Note> = Vec::new();
    for id in ids {
        let trail = steps.trail(steps.find(id)?);
        if !chosen.iter().any(|n| n.id == trail.id) {
            chosen.push(trail);
        }
    }
    let mut parts = Vec::new();
    for trail in chosen {
        let Some(step) = steps.step(trail) else {
            continue;
        };
        let count = steps.tree(trail).len() - 1;
        let ran = took(steps.newest(trail).saturating_sub(trail.created_at));
        let mut lines = vec![
            format!("## {}", step.title()),
            format!(
                "`{}` · {} · {} · {count} steps · took {ran}",
                short(&trail.id),
                step.anchor.name,
                steps.state(trail).text()
            ),
        ];
        let text: Vec<&str> = step.detail().collect();
        if !text.is_empty() {
            lines.push(String::new());
            lines.extend(text.iter().map(|l| l.to_string()));
        }
        if let Some(done) = steps.done(&trail.id) {
            lines.push(String::new());
            lines.push(format!("done: {}", done.content));
        }
        let rules = decisions.rules(&step.anchor.project, &trail.id);
        if !rules.is_empty() {
            lines.extend(["".into(), "**Rules**".into(), "".into()]);
            lines.extend(rules.iter().map(|(_, d)| format!("- {}", d.content)));
        }
        let mut notes = steps.under(&trail.id);
        if let Some(ask) = steps.twin(trail) {
            notes.remove(0);
            notes.splice(0..0, steps.under(&ask.id));
        }
        if !notes.is_empty() {
            lines.extend(["".into(), "**Steps**".into(), "".into()]);
            items(&steps, notes, "", &mut lines);
        }
        parts.push(lines.join("\n"));
    }
    Ok(parts.join("\n\n") + "\n")
}

/// Adds each step of `notes` as a list item after `indent`, with its lines and the steps under it.
fn items(steps: &Steps, notes: Vec<&Note>, indent: &str, lines: &mut Vec<String>) {
    for note in notes {
        let Some(step) = steps.step(note) else {
            continue;
        };
        let status = steps.status(&note.id);
        lines.push(format!(
            "{indent}- {} {}",
            step_look(status).0,
            step.title()
        ));
        let mut under: Vec<String> = step.detail().map(str::to_string).collect();
        under.extend(step.places.iter().map(|p| format!("at `{p}`")));
        if let Some(check) = &step.check {
            match status {
                Status::Done => under.push(format!("check passed `{check}`")),
                _ => under.push(format!("check `{check}`")),
            }
        }
        if let Some(done) = steps.done(&note.id) {
            under.push(format!("done: {}", done.content));
            under.extend(done.tag("changed").map(|c| format!("changed: {c}")));
        }
        if let Some(dropped) = steps.dropped(&note.id) {
            under.push(format!("dropped: {}", dropped.content));
        }
        lines.extend(under.iter().map(|l| format!("{indent}  - {l}")));
        items(steps, steps.under(&note.id), &format!("{indent}  "), lines);
    }
}
