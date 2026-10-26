//! Every trail of every project, one line an item, that `grep` and `search` read.

pub(crate) mod header;
pub(crate) mod kind;
pub(crate) mod row;

use crate::command::read::search::recalls::Recalls;
use crate::text::clock::date;
use celis_nostr::Note;
use celis_patterns::{Pattern, PatternEnd, PatternStore};
use celis_store::records::id::short;
use celis_store::Store;
use celis_trails::{Mark, Notes, Step, TrailStore};
use header::Header;
use kind::Kind;
use row::Row;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

/// Every trail of every project with its notes and steps, and each pattern, newest made first; with the
/// header of each trail.
#[derive(Default)]
pub(crate) struct View {
    rows: Vec<Row>,
    lines: HashMap<u64, usize>,
    headers: HashMap<String, Header>,
}

impl View {
    /// The view of `trails`, each pattern with the day `recalled` gives for its short id.
    pub fn recalled(trails: &Store, recalled: &Recalls) -> View {
        let (steps, patterns, notes) = (trails.steps(), trails.patterns(), trails.notes());
        let mut blocks: Vec<(u64, &str, Vec<Row>)> = Vec::new();
        for entry in patterns.every() {
            let note = entry.note;
            blocks.push((note.created_at, &note.id, pattern(entry, recalled)));
        }
        let mut headers = HashMap::new();
        for root in steps.roots() {
            headers.insert(short(root.id()), Header::of(root));
            let mut rows = vec![step(root, root, Kind::Trail)];
            rows.extend(noted(root, root, &notes));
            level(root, root, &notes, &mut rows);
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

/// Adds the steps under `parent` of `root` to `rows`, newest made first, each with its close, its
/// `notes` and then the steps under it.
fn level<'a>(root: Step<'a>, parent: Step<'a>, notes: &Notes, rows: &mut Vec<Row>) {
    for child in parent.children().into_iter().rev() {
        rows.push(step(root, child, Kind::Step));
        let row = |note: &Note, kind: Kind, text: &str| Row {
            trail: Some(short(root.id())),
            kind,
            day: date(note.created_at),
            project: root.record().anchor.name.clone(),
            id: short(child.id()),
            fields: vec![],
            text: flat(text),
        };
        match child.close() {
            Some((note, Mark::Done(celis_trails::Done { proof, changed, .. }))) => {
                rows.push(row(note, Kind::Proof, &proof));
                rows.extend(changed.map(|c| row(note, Kind::Changed, &c)));
            }
            Some((note, Mark::Drop { reason })) => rows.push(row(note, Kind::Dropped, &reason)),
            _ => {}
        }
        rows.extend(noted(root, child, notes));
        level(root, child, notes, rows);
    }
}

/// The rows of the notes on `step` of the trail `root`, oldest first.
fn noted(root: Step, step: Step, notes: &Notes) -> Vec<Row> {
    let on = notes.on(step.id()).iter();
    on.map(|note| Row {
        trail: Some(short(root.id())),
        kind: Kind::Note,
        day: date(note.created_at),
        project: root.record().anchor.name.clone(),
        id: short(step.id()),
        fields: vec![],
        text: flat(&note.content),
    })
    .collect()
}

/// The row of the step `step` of the trail `root`, as `kind`.
fn step(root: Step, step: Step, kind: Kind) -> Row {
    let record = step.record();
    let mut fields = vec![step.state().text().to_string()];
    let by: Vec<String> = step.revised().iter().map(|s| short(s.id())).collect();
    if !by.is_empty() {
        fields.push(format!("revised-by {}", by.join(",")));
    }
    fields.extend(record.places.iter().map(|p| format!("at:{p}")));
    Row {
        trail: Some(short(root.id())),
        kind,
        day: date(step.note().created_at),
        project: root.record().anchor.name.clone(),
        id: short(step.id()),
        fields,
        text: flat(&record.content),
    }
}

/// The rows of the pattern `entry`: its own, then its drop's reason when a drop names it.
fn pattern(entry: &Pattern, recalled: &Recalls) -> Vec<Row> {
    let (note, pattern) = (entry.note, &entry.record);
    let state = entry.end.map_or("live".to_string(), PatternEnd::text);
    let row = |kind: Kind, at: &Note, fields: Vec<String>, text: &str| Row {
        trail: None,
        kind,
        day: date(at.created_at),
        project: pattern.anchor.name.clone(),
        id: short(&note.id),
        fields,
        text: flat(text),
    };
    let mut fields = vec![state, format!("{{{}}}", pattern.scope.text())];
    let marks = [
        (!pattern.paths.is_empty(), "trigger"),
        (pattern.old_rule, "old-rule"),
    ];
    fields.extend(
        marks
            .into_iter()
            .filter(|(on, _)| *on)
            .map(|(_, mark)| mark.to_string()),
    );
    fields.extend(recalled.day(&note.id).map(|day| format!("recalled:{day}")));
    let mut rows = vec![row(Kind::Pattern, note, fields, &pattern.content)];
    if let Some(PatternEnd::Dropped(drop)) = entry.end {
        rows.push(row(Kind::Dropped, drop, vec![], &drop.content));
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
