//! The view `grep` searches: every trail of every project, one line an item.

use super::header::Header;
use super::kind::Kind;
use super::row::Row;
use crate::command::clock::date;
use celis_nostr::Note;
use celis_trails::id::short;
use celis_trails::{Decision, Mark, Scope, Step, Trails};
use std::collections::HashMap;

/// Every trail of every project with its rules and steps, and each decision with no trail, newest made
/// first; with the header of each trail.
#[derive(Default)]
pub(super) struct View {
    rows: Vec<Row>,
    lines: HashMap<String, usize>,
    headers: HashMap<String, Header>,
}

impl View {
    pub fn of(trails: &Trails) -> View {
        let (steps, decisions) = (trails.steps(), trails.decisions());
        let mut rules: HashMap<&str, Vec<Row>> = HashMap::new();
        let mut blocks: Vec<(u64, &str, Vec<Row>)> = Vec::new();
        for (note, decision, by) in decisions.every() {
            match &decision.scope {
                Scope::Trail(id) => {
                    let row = rule(note, decision, *by, Some(short(id)));
                    rules.entry(id.as_str()).or_default().push(row);
                }
                _ => blocks.push((
                    note.created_at,
                    &note.id,
                    vec![rule(note, decision, *by, None)],
                )),
            }
        }
        let mut headers = HashMap::new();
        for root in steps.roots() {
            headers.insert(short(root.id()), Header::of(root));
            let mut rows = vec![step(root, root, Kind::Trail)];
            rows.extend(rules.remove(root.id()).unwrap_or_default());
            level(root, root, &mut rows);
            blocks.push((root.note().created_at, root.id(), rows));
        }
        blocks.sort_by(|(a, x, _), (b, y, _)| (b, y).cmp(&(a, x)));
        let rows: Vec<Row> = blocks.into_iter().flat_map(|(.., rows)| rows).collect();
        let lines = rows
            .iter()
            .enumerate()
            .map(|(n, r)| (r.line(), n))
            .collect();
        View {
            rows,
            lines,
            headers,
        }
    }

    /// The lines, one a row.
    pub fn text(&self) -> String {
        let lines: Vec<String> = self.rows.iter().map(Row::line).collect();
        lines.join("\n")
    }

    /// The row `grep` printed as `line`, with any `<n>:` or `<n>-` it put before it.
    pub fn row<'a>(&self, line: &'a str) -> Option<(&'a str, &Row)> {
        let mut rest = line;
        loop {
            if let Some(&n) = self.lines.get(rest) {
                return Some((&line[..line.len() - rest.len()], &self.rows[n]));
            }
            let digits = rest.find(|c: char| !c.is_ascii_digit())?;
            match rest[digits..].chars().next() {
                Some(':' | '-') if digits > 0 => rest = &rest[digits + 1..],
                _ => return None,
            }
        }
    }

    /// The header of the trail `id`.
    pub fn header(&self, id: &str) -> Option<&Header> {
        self.headers.get(id)
    }
}

/// Adds the steps under `parent` of `root` to `rows`, newest made first, each with its close and then
/// the steps under it.
fn level<'a>(root: Step<'a>, parent: Step<'a>, rows: &mut Vec<Row>) {
    for child in parent.children().into_iter().rev() {
        rows.push(step(root, child, Kind::Step));
        let row = |note: &Note, kind: Kind, text: &str| Row {
            trail: Some(short(root.id())),
            kind,
            day: date(note.created_at),
            project: root.record().anchor.name.clone(),
            head: short(child.id()),
            text: flat(text),
        };
        match child.close() {
            Some((note, Mark::Done { proof, changed })) => {
                rows.push(row(note, Kind::Proof, &proof));
                rows.extend(changed.map(|c| row(note, Kind::Changed, &c)));
            }
            Some((note, Mark::Drop { reason })) => rows.push(row(note, Kind::Dropped, &reason)),
            _ => {}
        }
        level(root, child, rows);
    }
}

/// The row of the step `step` of the trail `root`, as `kind`.
fn step(root: Step, step: Step, kind: Kind) -> Row {
    let record = step.record();
    let mut head = vec![short(step.id()), step.state().text().to_string()];
    let by: Vec<String> = step.revised().iter().map(|s| short(s.id())).collect();
    if !by.is_empty() {
        head.push(format!("revised-by {}", by.join(",")));
    }
    head.extend(record.places.iter().map(|p| format!("at:{p}")));
    Row {
        trail: Some(short(root.id())),
        kind,
        day: date(step.note().created_at),
        project: root.record().anchor.name.clone(),
        head: head.join(" "),
        text: flat(&record.content),
    }
}

/// The row of the decision `note` holds, as a rule of `trail`, replaced by `by` when given.
fn rule(note: &Note, decision: &Decision, by: Option<&Note>, trail: Option<String>) -> Row {
    let state = by.map_or("live".to_string(), |b| {
        format!("replaced-by {}", short(&b.id))
    });
    Row {
        trail,
        kind: Kind::Rule,
        day: date(note.created_at),
        project: decision.anchor.name.clone(),
        head: format!("{} {state} {{{}}}", short(&note.id), decision.scope.short()),
        text: flat(&decision.content),
    }
}

/// `text` on one line.
fn flat(text: &str) -> String {
    let lines = text.split(['\n', '\r']).filter(|l| !l.is_empty());
    lines.collect::<Vec<_>>().join(" ")
}
