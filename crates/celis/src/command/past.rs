//! `celis past`: the past work of every project, one line each, which the retrieval log counts.

use super::clock::date;
use super::{Args, Command};
use celis_trails::{Failure, Past, RetrievalLog, Steps};
use celis_wire::Note;
use serde_json::json;
use std::io::Write;

impl Command {
    /// Prints each past work of every project, newest first, with its tags. Writes no record.
    pub(super) fn past(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        if !self.held()? {
            return Ok(());
        }
        let trails = self.trails()?;
        let steps = trails.steps();
        let all = Past::new(&steps).all();
        for note in &all {
            let Some(step) = steps.step(note) else {
                continue;
            };
            let day = date(note.created_at);
            let past = json!({"id": note.id, "created_at": note.created_at, "day": day,
                "project": step.anchor.name, "content": step.content, "places": step.places,
                "state": steps.state(steps.trail(note)).text(),
                "grade": steps.grade(note).as_ref().map(|g| g.value())});
            let line = format!("{} · {day}", head(note, &steps));
            super::show(out, args.json, "past", past, &line)?;
        }
        let _ = RetrievalLog::at(&self.home()?).listed(all.len(), trails.now);
        Ok(())
    }
}

/// The line of the past work `note`: its id, project, title and place, then its grade, else its state.
fn head(note: &Note, steps: &Steps) -> String {
    let Some(step) = steps.step(note) else {
        return String::new();
    };
    let grade = steps.grade(note);
    let how = grade.map_or_else(|| steps.state(note).text().to_string(), |g| g.line());
    format!("{} · {how}", step.line(note))
}
