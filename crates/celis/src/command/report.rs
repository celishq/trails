//! `celis report`: the trails that ids name, as Markdown for a pull request.

use super::tree::look;
use super::{Args, Command};
use celis_trails::id::short;
use celis_trails::trail::grade::took;
use celis_trails::{Failure, Step, Trails};
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
    let mut chosen: Vec<Step> = Vec::new();
    for id in ids {
        let trail = steps.find(id)?.trail();
        if !chosen.iter().any(|s| s.id() == trail.id()) {
            chosen.push(trail);
        }
    }
    let mut parts = Vec::new();
    for trail in chosen {
        let step = trail.record();
        let count = trail.tree().len() - 1;
        let ran = took(trail.newest().saturating_sub(trail.note().created_at));
        let mut lines = vec![
            format!("## {}", step.title()),
            format!(
                "`{}` · {} · {} · {count} steps · took {ran}",
                short(trail.id()),
                step.anchor.name,
                trail.state().text()
            ),
        ];
        let text: Vec<&str> = step.detail().collect();
        if !text.is_empty() {
            lines.push(String::new());
            lines.extend(text.iter().map(|l| l.to_string()));
        }
        if let Some(done) = trail.done() {
            lines.push(String::new());
            lines.push(format!("done: {}", done.content));
        }
        let rules = decisions.rules(&step.anchor.project, trail.id());
        if !rules.is_empty() {
            lines.extend(["".into(), "**Rules**".into(), "".into()]);
            lines.extend(rules.iter().map(|(_, d)| format!("- {}", d.content)));
        }
        let mut parts_of = trail.children();
        if let Some(ask) = trail.twin() {
            parts_of.remove(0);
            parts_of.splice(0..0, ask.children());
        }
        if !parts_of.is_empty() {
            lines.extend(["".into(), "**Steps**".into(), "".into()]);
            items(parts_of, "", &mut lines);
        }
        parts.push(lines.join("\n"));
    }
    Ok(parts.join("\n\n") + "\n")
}

/// Adds each step of `steps` as a list item after `indent`, with its lines and the steps under it.
fn items(steps: Vec<Step>, indent: &str, lines: &mut Vec<String>) {
    for at in steps {
        let step = at.record();
        let state = at.state();
        lines.push(format!(
            "{indent}- {} {}",
            look(state, false).0,
            step.title()
        ));
        let mut under: Vec<String> = step.detail().map(str::to_string).collect();
        under.extend(step.places.iter().map(|p| format!("at `{p}`")));
        if let Some(check) = &step.check {
            match state.reached() {
                true => under.push(format!("check passed `{check}`")),
                false => under.push(format!("check `{check}`")),
            }
        }
        if let Some(done) = at.done() {
            under.push(format!("done: {}", done.content));
            under.extend(done.tag("changed").map(|c| format!("changed: {c}")));
        }
        if let Some(dropped) = at.dropped() {
            under.push(format!("dropped: {}", dropped.content));
        }
        lines.extend(under.iter().map(|l| format!("{indent}  - {l}")));
        items(at.children(), &format!("{indent}  "), lines);
    }
}
