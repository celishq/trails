//! The view `grep` searches: every trail of every project, one line an item.

use super::kind::Kind;
use crate::command::clock::date;
use crate::command::line;
use celis_nostr::Note;
use celis_trails::id::short;
use celis_trails::{Decision, Mark, Scope, Step, Trails};
use std::collections::HashMap;

/// One line of the view: the trail it belongs to, `None` for a decision with no trail, and its kind.
pub(super) struct Row {
    pub trail: Option<String>,
    pub kind: Kind,
    pub line: String,
    body: usize,
}

impl Row {
    /// The line after its day, its trail and, when it has a trail, its project.
    pub fn body(&self) -> &str {
        &self.line[self.body..]
    }

    /// The row of `trail` and `kind` whose line is `lead`, a space, then `body`.
    fn of(trail: Option<String>, kind: Kind, lead: &str, body: &str) -> Row {
        Row {
            trail,
            kind,
            line: format!("{lead} {body}"),
            body: lead.len() + 1,
        }
    }
}

/// Every trail of every project with its rules and steps, and each decision with no trail, newest made
/// first; with the header of each trail.
#[derive(Default)]
pub(super) struct View {
    rows: Vec<Row>,
    lines: HashMap<String, usize>,
    titles: HashMap<String, (String, String)>,
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
        let mut titles = HashMap::new();
        for root in steps.roots() {
            let head = (line::step(root), date(root.note().created_at));
            titles.insert(short(root.id()), head);
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
            .map(|(n, r)| (r.line.clone(), n))
            .collect();
        View {
            rows,
            lines,
            titles,
        }
    }

    /// The lines, one a row.
    pub fn text(&self) -> String {
        let lines: Vec<&str> = self.rows.iter().map(|r| r.line.as_str()).collect();
        lines.join("\n")
    }

    /// The row `grep` printed as `line`, after any `<n>:` or `<n>-` it put before it.
    pub fn row(&self, line: &str) -> Option<&Row> {
        let mut rest = line;
        loop {
            if let Some(&n) = self.lines.get(rest) {
                return Some(&self.rows[n]);
            }
            let digits = rest.find(|c: char| !c.is_ascii_digit())?;
            match rest[digits..].chars().next() {
                Some(':' | '-') if digits > 0 => rest = &rest[digits + 1..],
                _ => return None,
            }
        }
    }

    /// `<id, first 8> [<project name>] <title> · <yyyy-mm-dd>` of the trail `id`.
    pub fn title(&self, id: &str) -> String {
        self.titles
            .get(id)
            .map_or_else(|| id.to_string(), |(head, day)| format!("{head} · {day}"))
    }

    /// `<id, first 8> [<project name>] <title>` of the trail `id`, its title cut at `chars` characters.
    pub fn short(&self, id: &str, chars: usize) -> String {
        let Some((head, _)) = self.titles.get(id) else {
            return id.to_string();
        };
        let start = head.find("] ").map_or(0, |n| n + 2);
        match head[start..].char_indices().nth(chars) {
            Some((end, _)) => format!("{}…", &head[..start + end]),
            None => head.clone(),
        }
    }
}

/// Adds the steps under `parent` of `root` to `rows`, newest made first, each with its close and then
/// the steps under it.
fn level<'a>(root: Step<'a>, parent: Step<'a>, rows: &mut Vec<Row>) {
    for child in parent.children().into_iter().rev() {
        rows.push(step(root, child, Kind::Step));
        let row = |note: &Note, kind: Kind, text: &str| {
            let body = format!("{} {} {}", kind.text(), short(child.id()), flat(text));
            Row::of(Some(short(root.id())), kind, &lead(root, note), &body)
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
    let mut fields = vec![step.state().text().to_string()];
    let by: Vec<String> = step.revised().iter().map(|s| short(s.id())).collect();
    if !by.is_empty() {
        fields.push(format!("revised-by {}", by.join(",")));
    }
    fields.extend(record.places.iter().map(|p| format!("at:{p}")));
    let body = format!(
        "{} {} {} {}",
        kind.text(),
        short(step.id()),
        fields.join(" "),
        flat(&record.content)
    );
    Row::of(
        Some(short(root.id())),
        kind,
        &lead(root, step.note()),
        &body,
    )
}

/// The row of the decision `note` holds, as a rule of `trail`, replaced by `by` when given.
fn rule(note: &Note, decision: &Decision, by: Option<&Note>, trail: Option<String>) -> Row {
    let state = by.map_or("live".to_string(), |b| {
        format!("replaced-by {}", short(&b.id))
    });
    let (day, name) = (date(note.created_at), format!("[{}]", decision.anchor.name));
    let body = format!(
        "rule {} {state} {{{}}} {}",
        short(&note.id),
        decision.scope.short(),
        flat(&decision.content)
    );
    match &trail {
        Some(id) => Row::of(
            trail.clone(),
            Kind::Rule,
            &format!("{day} {id} {name}"),
            &body,
        ),
        None => Row::of(
            None,
            Kind::Rule,
            &format!("{day} -"),
            &format!("{name} {body}"),
        ),
    }
}

/// `<day of note> <trail id, first 8> [<project name>]` of a line of the trail `root`.
fn lead(root: Step, note: &Note) -> String {
    let name = &root.record().anchor.name;
    format!("{} {} [{name}]", date(note.created_at), short(root.id()))
}

/// `text` on one line.
fn flat(text: &str) -> String {
    let lines = text.split(['\n', '\r']).filter(|l| !l.is_empty());
    lines.collect::<Vec<_>>().join(" ")
}
