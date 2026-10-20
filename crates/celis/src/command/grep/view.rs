//! The view `grep` searches: every trail of every project, one line an item.

use super::header::Header;
use super::kind::Kind;
use super::row::Row;
use crate::text::clock::date;
use celis_nostr::Note;
use celis_trails::records::id::short;
use celis_trails::{Mark, RuleEnd, RuleEntry, Scope, Step, Trails};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

/// Every trail of every project with its rules and steps, and each rule with no trail, newest made
/// first; with the header of each trail.
#[derive(Default)]
pub(super) struct View {
    rows: Vec<Row>,
    lines: HashMap<u64, usize>,
    headers: HashMap<String, Header>,
}

impl View {
    pub fn of(trails: &Trails) -> View {
        let (steps, rules) = (trails.steps(), trails.rules());
        let mut by_trail: HashMap<&str, Vec<Row>> = HashMap::new();
        let mut blocks: Vec<(u64, &str, Vec<Row>)> = Vec::new();
        for entry in rules.every() {
            let note = entry.note;
            match &entry.rule.scope {
                Scope::Trail(id) => {
                    let rows = rule(entry, Some(short(id)));
                    by_trail.entry(id.as_str()).or_default().extend(rows);
                }
                _ => blocks.push((note.created_at, &note.id, rule(entry, None))),
            }
        }
        let mut headers = HashMap::new();
        for root in steps.roots() {
            headers.insert(short(root.id()), Header::of(root));
            let mut rows = vec![step(root, root, Kind::Trail)];
            rows.extend(by_trail.remove(root.id()).unwrap_or_default());
            level(root, root, &mut rows);
            blocks.push((root.note().created_at, root.id(), rows));
        }
        blocks.sort_by(|(a, x, _), (b, y, _)| (b, y).cmp(&(a, x)));
        let rows: Vec<Row> = blocks.into_iter().flat_map(|(.., rows)| rows).collect();
        let lines = rows
            .iter()
            .enumerate()
            .map(|(n, r)| (hash(&r.line()), n))
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
            let row = self.lines.get(&hash(rest)).map(|&n| &self.rows[n]);
            if let Some(row) = row.filter(|r| r.line() == rest) {
                return Some((&line[..line.len() - rest.len()], row));
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
    let mut head = vec![short(step.id()), step.state_word().to_string()];
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

/// The rows of the rule `entry`, as a rule of `trail`: its own, then its drop's reason when a drop
/// names it.
fn rule(entry: &RuleEntry, trail: Option<String>) -> Vec<Row> {
    let (note, rule) = (entry.note, &entry.rule);
    let state = entry.end.map_or("live".to_string(), RuleEnd::text);
    let row = |kind: Kind, at: &Note, head: String, text: &str| Row {
        trail: trail.clone(),
        kind,
        day: date(at.created_at),
        project: rule.anchor.name.clone(),
        head,
        text: flat(text),
    };
    let head = format!("{} {state} {{{}}}", short(&note.id), rule.scope.short());
    let mut rows = vec![row(Kind::Rule, note, head, &rule.content)];
    if let Some(RuleEnd::Dropped(drop)) = entry.end {
        rows.push(row(Kind::Dropped, drop, short(&note.id), &drop.content));
    }
    rows
}

/// The hash of `line`, by which the view finds its row.
fn hash(line: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    line.hash(&mut hasher);
    hasher.finish()
}

/// `text` on one line.
fn flat(text: &str) -> String {
    let lines = text.split(['\n', '\r']).filter(|l| !l.is_empty());
    lines.collect::<Vec<_>>().join(" ")
}
