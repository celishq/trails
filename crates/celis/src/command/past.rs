//! `celis past`: the working folder's project's active trails and decisions, then the past work of every
//! project, one line each.

use super::clock::date;
use super::{Args, Command};
use celis_trails::{Decisions, Failure, Project, Scope, Step, Trail, Trails};
use serde_json::json;
use std::io::Write;

impl Command {
    /// Prints what `list` prints, else a line that says there is nothing. Writes no record.
    pub(super) fn past(
        &self,
        args: &Args,
        out: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        let mut listed: Vec<u8> = Vec::new();
        if self.held()? {
            self.list(&self.trails()?, args, &mut listed, project)?;
        }
        if listed.is_empty() && !args.json {
            return super::put(out, "No active trails, decisions or past work yet.");
        }
        out.write_all(&listed)
            .map_err(anyhow::Error::from)
            .map_err(Failure::from)
    }

    /// Prints the active trails of `project`, oldest first, each with its rules; then its live decisions at
    /// each `--at` place, then its `project` and `me` ones; then each past work of every project, newest
    /// first, with the rules of the trail it stands for.
    pub(super) fn list(
        &self,
        trails: &Trails,
        args: &Args,
        out: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        let steps = trails.steps();
        let decisions = trails.decisions();
        for step in steps.trails(&project.id) {
            if step.state().active() {
                trail(step, &decisions, args.json, out)?;
            }
        }
        let placed = args
            .places
            .iter()
            .flat_map(|p| decisions.at(&project.id, &project.place(p, &self.folder)));
        let rest = decisions
            .live(&project.id)
            .into_iter()
            .filter(|(_, d)| !matches!(d.scope, Scope::Trail(_) | Scope::At(_)));
        let mut shown: Vec<String> = Vec::new();
        for (note, decision) in placed.chain(rest) {
            if shown.contains(&note.id) {
                continue;
            }
            shown.push(note.id.clone());
            let line = super::line::decision(note, &decision);
            super::show(
                out,
                args.json,
                "decision",
                super::json::decision(note, &decision),
                &line,
            )?;
        }
        for step in steps.past() {
            let (note, record) = (step.note(), step.record());
            let day = date(note.created_at);
            let rules = held(step).map_or_else(Vec::new, |t| {
                decisions.rules(&record.anchor.project, &t.note().id)
            });
            let listed: Vec<_> = rules
                .iter()
                .map(|(r, d)| super::json::decision(r, d))
                .collect();
            let past = json!({"id": note.id, "created_at": note.created_at, "day": day,
                "project": record.anchor.name, "content": record.content, "places": record.places,
                "state": step.trail().state().text(),
                "grade": step.grade(), "rules": listed});
            let line = format!("{} · {day}", head(step));
            super::show(out, args.json, "past", past, &line)?;
            if !args.json {
                for (rule, decision) in &rules {
                    super::put(out, &super::line::rule(rule, decision))?;
                }
            }
        }
        Ok(())
    }
}

/// Prints the trail `trail` with its rules.
fn trail(
    trail: Step,
    decisions: &Decisions,
    json: bool,
    out: &mut dyn Write,
) -> Result<(), Failure> {
    let (note, step) = (trail.note(), trail.record());
    let rules = decisions.rules(&step.anchor.project, &note.id);
    if json {
        let rules: Vec<_> = rules
            .iter()
            .map(|(r, d)| super::json::decision(r, d))
            .collect();
        let trail = json!({"id": note.id, "created_at": note.created_at,
            "project": step.anchor.project, "content": step.content, "session": step.session,
            "rules": rules});
        return super::put(out, &json!({ "trail": trail }).to_string());
    }
    super::put(out, &super::line::step(note, step))?;
    for (rule, decision) in rules {
        super::put(out, &super::line::rule(rule, &decision))?;
    }
    Ok(())
}

/// The trail whose rules the past work `step` holds: the trail itself, or the trail of its own text.
fn held(step: Step) -> Option<Step> {
    if step.is_trail() {
        return Some(step);
    }
    let trail = step.trail();
    let first = Trail::new(trail).first()?;
    (first.note().id == step.note().id).then_some(trail)
}

/// The line of the past work `step`: its id, project, title and place, then its grade, else its state.
fn head(step: Step) -> String {
    let how = step.grade().map_or_else(
        || step.state().text().to_string(),
        |g| super::line::grade(&g),
    );
    format!("{} · {how}", super::line::step(step.note(), step.record()))
}
