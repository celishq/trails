//! `celis past`: the working folder's project's active trails and decisions, then the past work of every
//! project, one line each.

use super::clock::date;
use super::{Args, Command};
use celis_trails::{Decisions, Failure, Project, Scope, Step, Trails};
use serde_json::json;
use std::io::Write;

impl Command {
    /// Prints what `list` prints. Writes no record.
    pub(super) fn past(
        &self,
        args: &Args,
        out: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        if !self.held()? {
            return Ok(());
        }
        self.list(&self.trails()?, args, out, project)
    }

    /// Prints the active trails of `project`, oldest first, each with its rules; then its live decisions at
    /// each `--at` place, then its `project` and `me` ones; then each past work of every project, newest
    /// first.
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
            let line = decision.line(note);
            super::show(out, args.json, "decision", decision.json(note), &line)?;
        }
        for step in steps.past() {
            let (note, record) = (step.note(), step.record());
            let day = date(note.created_at);
            let past = json!({"id": note.id, "created_at": note.created_at, "day": day,
                "project": record.anchor.name, "content": record.content, "places": record.places,
                "state": step.trail().state().text(),
                "grade": step.grade().as_ref().map(|g| g.value())});
            let line = format!("{} · {day}", head(step));
            super::show(out, args.json, "past", past, &line)?;
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
        let rules: Vec<_> = rules.iter().map(|(r, d)| d.json(r)).collect();
        let trail = json!({"id": note.id, "created_at": note.created_at,
            "project": step.anchor.project, "content": step.content, "session": step.session,
            "rules": rules});
        return super::put(out, &json!({ "trail": trail }).to_string());
    }
    super::put(out, &step.line(note))?;
    for (rule, decision) in rules {
        super::put(out, &decision.rule(rule))?;
    }
    Ok(())
}

/// The line of the past work `step`: its id, project, title and place, then its grade, else its state.
fn head(step: Step) -> String {
    let how = step
        .grade()
        .map_or_else(|| step.state().text().to_string(), |g| g.line());
    format!("{} · {how}", step.record().line(step.note()))
}
