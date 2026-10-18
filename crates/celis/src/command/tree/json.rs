//! The tree as JSON: an object a line for each trail and step.

use super::row::Row;
use super::Tree;
use crate::progress::Grade;
use celis_trails::Step;
use serde_json::json;

impl Tree<'_> {
    /// Adds the object of the step `note`, named `key`, then each step under it as its own.
    pub(super) fn json(&self, note: Step, key: &str, lines: &mut Vec<Row>) {
        let step = note.record();
        let mut head = json!({"id": note.id(), "created_at": note.note().created_at,
            "project": step.anchor.project, "content": step.content,
            "links": crate::text::json::links(step), "places": step.places, "session": step.session});
        self.closing_value(&mut head, note, self.work(note));
        let reworked: Vec<&str> = note.reworked().iter().map(|s| s.id()).collect();
        head["revised"] = json!(reworked);
        lines.push(Row::Text(json!({ key: head }).to_string().into()));
        self.objects(note.children(), lines);
    }

    /// Adds the object of each step of `notes`, each followed by its own.
    fn objects(&self, notes: Vec<Step>, lines: &mut Vec<Row>) {
        for note in notes {
            let mut value = crate::text::json::step(note.note(), note.record());
            self.closing_value(&mut value, note, note);
            lines.push(Row::Text(json!({ "step": value }).to_string().into()));
            self.objects(note.children(), lines);
        }
    }

    /// Adds to `value` the state of `note`, and the proof, change, grade, reuse and reason of `work`.
    fn closing_value(&self, value: &mut serde_json::Value, note: Step, work: Step) {
        let done = work.done();
        value["state"] = json!(note.state().text());
        value["proof"] = json!(done.map(|d| d.content.clone()));
        value["changed"] = json!(work.changed());
        value["grade"] = json!(Grade::of(work));
        let by: Vec<&str> = work.revised().iter().map(|s| s.id()).collect();
        value["revised_by"] = json!(by);
        value["followed"] = json!(self.reuse.followers(work));
        value["reason"] = json!(work.dropped().map(|d| d.content.clone()));
    }
}
