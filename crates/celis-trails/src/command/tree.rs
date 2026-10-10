//! `celis tree`: a trail with its state, its rules and the steps under it.

use super::{Args, Command};
use crate::id::short;
use crate::{Decisions, Failure, Project, Step, Steps};
use celis_wire::Note;
use serde_json::json;
use std::io::Write;

impl Command {
    /// Prints the step `args` names with every step under it, or with no id every trail of `project`,
    /// newest first. Writes nothing.
    pub(super) fn tree(
        &self,
        args: &Args,
        out: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        let id = args.words.get(1);
        if !self.held()? {
            return id.map_or(Ok(()), |id| Err(Failure::no_step(id)));
        }
        let store = self.store()?;
        let records = store.records();
        let steps = Steps { records: &records };
        let decisions = Decisions {
            records: &records,
            account: store.account().pubkey(),
        };
        let chosen: Vec<&Note> = match id {
            Some(id) => vec![steps.find(id)?],
            None => steps.trails(&project.id).into_iter().rev().collect(),
        };
        let tree = Tree {
            steps: &steps,
            decisions: &decisions,
        };
        for note in chosen {
            tree.print(note, "trail", args.json, out)?;
        }
        Ok(())
    }
}

/// The steps and decisions a tree is printed from.
pub(super) struct Tree<'a> {
    pub steps: &'a Steps<'a>,
    pub decisions: &'a Decisions<'a>,
}

impl Tree<'_> {
    /// Prints the step `note` with its state, its proof and grade when done, the trail it follows, and its
    /// rules, then the steps under it. With `json`, its object is named `key`.
    pub fn print(
        &self,
        note: &Note,
        key: &str,
        json: bool,
        out: &mut dyn Write,
    ) -> Result<(), Failure> {
        let Some(step) = Step::read(note) else {
            return Ok(());
        };
        let state = self.steps.state(note);
        let proof = self.steps.done(&note.id).map(|d| d.content.clone());
        let grade = self.steps.grade(note);
        let rules = self.decisions.rules(&step.anchor.project, &note.id);
        if json {
            let head = json!({"id": note.id, "created_at": note.created_at,
                "project": step.anchor.project, "content": step.content, "state": state,
                "follows": step.follows, "proof": proof, "grade": grade.as_ref().map(|g| g.value())});
            super::put(out, &json!({ key: head }).to_string())?;
        } else {
            super::put(out, &format!("{} · {state}", step.line(note)))?;
            if let (Some(proof), Some(grade)) = (&proof, &grade) {
                super::put(out, &format!("    done {proof}"))?;
                super::put(out, &format!("    {}", grade.line()))?;
            }
            if let Some(past) = step
                .follows
                .as_deref()
                .and_then(|id| self.steps.find(id).ok())
            {
                let first = past.content.lines().next().unwrap_or("");
                super::put(out, &format!("  follows {} {first}", short(&past.id)))?;
            }
            for (rule, decision) in rules {
                super::put(out, &decision.rule(rule))?;
            }
        }
        self.under(&note.id, 0, json, out)
    }

    /// Prints the steps under the step `id` at `depth`, each followed by its own.
    fn under(
        &self,
        id: &str,
        depth: usize,
        json: bool,
        out: &mut dyn Write,
    ) -> Result<(), Failure> {
        for note in self.steps.under(id) {
            let Some(step) = Step::read(note) else {
                continue;
            };
            let proof = self.steps.done(&note.id).map(|d| d.content.clone());
            let grade = self.steps.grade(note);
            let status = self.steps.status(&note.id);
            if json {
                let mut value = step.value(note);
                value["state"] = json!(status);
                value["proof"] = json!(proof);
                value["grade"] = json!(grade.as_ref().map(|g| g.value()));
                super::put(out, &json!({ "step": value }).to_string())?;
            } else {
                let pad = "  ".repeat(depth);
                let mark = match status {
                    "done" => "✓",
                    "working" => "◐",
                    _ => "○",
                };
                super::put(out, &format!("{pad}{mark} {}", step.line(note)))?;
                if let (Some(proof), Some(grade)) = (&proof, &grade) {
                    super::put(out, &format!("{pad}    done {proof}"))?;
                    super::put(out, &format!("{pad}    {}", grade.line()))?;
                }
            }
            self.under(&note.id, depth + 1, json, out)?;
        }
        Ok(())
    }
}
