//! Every trail of every project, one line an item, that `grep` reads.

pub(crate) mod header;
pub(crate) mod kind;
pub(crate) mod row;

use crate::text::clock::date;
use celis_nostr::Note;
use celis_patterns::{Example, Pattern, PatternEnd, PatternStore};
use celis_store::records::id::short;
use celis_store::Store;
use celis_trails::{Mark, Notes, Step, Steps, TrailStore};
use header::Header;
use kind::Kind;
use row::Row;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

/// Every trail of every project with its notes, prompts and steps, and each pattern, newest made first;
/// with the header of each trail.
#[derive(Default)]
pub(crate) struct View {
    rows: Vec<Row>,
    lines: HashMap<u64, usize>,
    headers: HashMap<String, Header>,
}

impl View {
    /// The view of `trails`.
    pub fn of(trails: &Store) -> View {
        let (steps, patterns, notes) = (trails.steps(), trails.patterns(), trails.notes());
        let mut blocks: Vec<(u64, &str, Vec<Row>)> = Vec::new();
        for entry in patterns.every() {
            let note = entry.note;
            blocks.push((note.created_at, &note.id, pattern(entry, &steps, trails)));
        }
        let mut prompts: HashMap<String, Vec<(&Note, String)>> = HashMap::new();
        for record in trails.records() {
            if let Some((on, Mark::Prompt { text })) = Mark::read(record) {
                prompts.entry(on).or_default().push((record, text));
            }
        }
        let mut headers = HashMap::new();
        for root in steps.roots() {
            headers.insert(short(root.id()), Header::of(root));
            let mut rows = vec![step(root, root, Kind::Trail)];
            rows.extend(noted(root, root, &notes));
            let mut asked: Vec<_> = prompts.get(root.id()).into_iter().flatten().collect();
            asked.sort_by_key(|(note, _)| (note.created_at, &note.id));
            rows.extend(
                asked
                    .into_iter()
                    .map(|(note, text)| prompt(root, note, text)),
            );
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

/// The row of the prompt `note`, that says `text`, on the trail `root`.
fn prompt(root: Step, note: &Note, text: &str) -> Row {
    Row {
        trail: Some(short(root.id())),
        kind: Kind::Prompt,
        day: date(note.created_at),
        project: root.record().anchor.name.clone(),
        id: short(root.id()),
        fields: vec![],
        text: flat(text),
    }
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

/// The rows of the pattern `entry`: its own, with how many asks of `steps` recalled it and steps used it
/// when not none, how many it misled when not none, and the day of the newest recall; then each of its examples, then its drop's reason when a drop
/// names it.
fn pattern(entry: &Pattern, steps: &Steps, trails: &Store) -> Vec<Row> {
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
    if pattern.old_rule {
        fields.push("old-rule".to_string());
    }
    let recalls = steps.recalls(&note.id);
    let uses = steps.all().into_iter();
    let uses = uses.filter(|s| s.record().uses.contains(&note.id));
    let counts = (recalls.len(), uses.count());
    if counts != (0, 0) {
        fields.push(format!("recalls:{} uses:{}", counts.0, counts.1));
    }
    let misled = steps.misled(&note.id).len();
    if misled > 0 {
        fields.push(format!("misled:{misled}"));
    }
    let newest = recalls.first().map(|ask| date(ask.note().created_at));
    fields.extend(newest.map(|day| format!("recalled:{day}")));
    let mut rows = vec![row(Kind::Pattern, note, fields, &pattern.content)];
    for (side, example) in pattern.examples.sides() {
        let kind = match side {
            "good" => Kind::Good,
            _ => Kind::Bad,
        };
        let text = match example {
            Example::Step(id) => {
                let Ok(step) = steps.find(id) else { continue };
                let done = step.done();
                let proof = done.as_ref().and_then(|d| d.proof.lines().next());
                format!(
                    "{} {} · {}",
                    short(step.id()),
                    step.title(),
                    proof.unwrap_or("")
                )
            }
            Example::File(file) => {
                format!("{} · {}", file.place, crate::text::line::blob(file, trails))
            }
            Example::Url(url) => url.to_string(),
        };
        rows.push(row(kind, note, vec![], &text));
    }
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
