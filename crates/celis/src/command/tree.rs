//! `celis tree`: a trail with its state, its rules and the steps under it.

use super::clock::{clock, day, midnight};
use super::clock::{saved, stride, took};
use super::{Args, Command};
use celis_trails::id::short;
use celis_trails::trail::steps::LIFE;
use celis_trails::{
    Decisions, Failure, Grade, Reuse, State, Step, StepRecord, Steps, Trail, Trails,
};
use serde_json::json;
use std::cell::Cell;
use std::io::{IsTerminal, Write};

impl Command {
    /// Prints the step `args` names with every step under it, or with no id each trail of every project
    /// that is active or has a record from today, newest first, else `EMPTY`; on a terminal with no
    /// `--json`, again on each change. Writes no record.
    pub(super) fn tree(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let id = args.words.get(1);
        if args.all && id.is_some() {
            return Err(Failure::usage("tree takes an id or --all, not both"));
        }
        if std::io::stdout().is_terminal() && !args.json {
            return self.watch(args, out);
        }
        let tree = match (self.held()?, id) {
            (true, _) => self.frame(&self.trails()?, args, self.columns())?.0,
            (false, Some(id)) => return Err(Failure::no_step(id)),
            (false, None) => String::new(),
        };
        let tree = match tree.is_empty() && id.is_none() && !args.all && !args.json {
            true => format!("{EMPTY}\n"),
            false => tree,
        };
        write!(out, "{tree}").map_err(anyhow::Error::from)?;
        Ok(())
    }

    /// The tree of `trails` that `args` names, `width` wide, and the earliest time a time it shows
    /// changes.
    pub(super) fn frame(
        &self,
        trails: &Trails,
        args: &Args,
        width: Option<usize>,
    ) -> Result<(String, Option<u64>), Failure> {
        let id = args.words.get(1);
        let steps = trails.steps();
        let decisions = trails.decisions();
        let reuse = Reuse::new(&steps);
        let chosen: Vec<Step> = match (id, args.all) {
            (Some(id), _) => vec![steps.find(id)?],
            (None, true) => steps.roots().into_iter().rev().collect(),
            (None, false) => today(&steps, trails.now),
        };
        let tree = Tree {
            steps: &steps,
            decisions: &decisions,
            reuse: &reuse,
            colour: self.colour(),
            full: args.full,
            fold: id.is_none() && !args.full,
            all: args.all,
            named: id.is_none(),
            width,
            next: Cell::default(),
        };
        let mut printed = Vec::new();
        tree.print(&chosen, "trail", args.json, &mut printed)?;
        let next = tree.next.get();
        Ok((String::from_utf8_lossy(&printed).into_owned(), next))
    }
}

/// Each trail of every project that is active or has a record from the day of `now`, newest first.
pub(super) fn today<'a>(steps: &'a Steps<'a>, now: u64) -> Vec<Step<'a>> {
    let midnight = midnight(now);
    let roots = steps.roots().into_iter().rev();
    roots
        .filter(|s| s.state().active() || s.newest() >= midnight)
        .collect()
}

/// What `tree` with no id prints while it has no trail to print.
pub(super) const EMPTY: &str = "No active trails. Run `celis tree --all` to see every trail.";

/// The steps and decisions a tree is printed from, and how: in colour; full, with the lines under each
/// step; fold, with each trail that is not active as one line; all, by day; named, with each trail's
/// project; else each line cut to `width`. `next` holds the earliest time a time it printed changes.
pub(super) struct Tree<'a> {
    pub steps: &'a Steps<'a>,
    pub decisions: &'a Decisions<'a>,
    pub reuse: &'a Reuse<'a>,
    pub colour: bool,
    pub full: bool,
    pub fold: bool,
    pub all: bool,
    pub named: bool,
    pub width: Option<usize>,
    pub next: Cell<Option<u64>>,
}

/// A line of the tree: text as it is, or a step's row whose details line up with the other rows'.
enum Line {
    Text(String),
    Row {
        lead: String,
        title: String,
        cells: [Option<String>; 4],
    },
}

impl Tree<'_> {
    /// Prints each step of `notes` as `trail` gives it, the details of all their rows in the same columns;
    /// with `all`, each day's steps after a line with the day.
    pub fn print(
        &self,
        notes: &[Step],
        key: &str,
        json: bool,
        out: &mut dyn Write,
    ) -> Result<(), Failure> {
        let mut lines = vec![];
        let mut last = None;
        let name = notes
            .iter()
            .map(|s| s.record().anchor.name.chars().count())
            .max()
            .unwrap_or(0);
        for &note in notes {
            let day = day(note.note().created_at);
            if self.all && !json && last.as_ref() != Some(&day) {
                lines.push(Line::Text(self.paint(BOLD, &day)));
                last = Some(day);
            }
            self.trail(note, key, json, name, &mut lines);
        }
        self.show(lines, out)
    }

    /// Adds the step `note` with its state, its proof and grade when done, then its rules or an ask's
    /// trail's rules and the steps under it as branches; a first ask with its text stands on its line, its
    /// project `name` wide. With `json`, its object is `key`.
    fn trail(&self, note: Step, key: &str, json: bool, name: usize, lines: &mut Vec<Line>) {
        let step = note.record();
        let state = note.state();
        let work = Trail::new(note).work();
        let done = work.done();
        let grade = work.grade();
        let reason = work.dropped().map(|d| d.content.clone());
        if json {
            let head = json!({"id": note.id(), "created_at": note.note().created_at,
                "project": step.anchor.project, "content": step.content, "state": state.text(),
                "links": super::json::links(step), "places": step.places, "session": step.session,
                "proof": done.map(|d| d.content.clone()),
                "changed": done.and_then(|d| d.tag("changed")),
                "grade": grade,
                "reused": self.reuse.down(note).len(), "reason": reason});
            lines.push(Line::Text(json!({ key: head }).to_string()));
            return self.under(note.children(), "", json, lines);
        }
        let folded = self.fold && !state.active();
        let count = note.tree().len() - 1;
        let steps = match (folded, count) {
            (false, _) | (true, 0) => None,
            (true, 1) => Some("1 step".to_string()),
            (true, n) => Some(format!("{n} steps")),
        };
        let (mark, colour) = look(state, true);
        let project = match self.named {
            true => format!(
                "{} ",
                self.paint(DIM, &format!("{:<name$}", step.anchor.name))
            ),
            false => String::new(),
        };
        let lead = format!("{} {} {project}", self.paint(colour, mark), self.id(note));
        let ran = match state.active() {
            true => {
                self.until(note.newest() + LIFE);
                self.since(note.note().created_at)
            }
            false => took(note.newest().saturating_sub(note.note().created_at)),
        };
        let column = visible(&lead);
        lines.push(Line::Row {
            lead,
            title: self.paint(BOLD, step.title()),
            cells: [
                step.at().map(last),
                steps,
                Some(ran),
                Some(self.clock(note.note().created_at)),
            ],
        });
        self.closing(step, done, &grade, &reason, ("", column), lines);
        self.reused(work, ("", column), lines);
        if folded {
            return;
        }
        let owner = match note.is_ask() {
            true => note.trail(),
            false => note,
        };
        let notes = Trail::new(note).parts();
        let rules = self.decisions.rules(&step.anchor.project, owner.id());
        let count = rules.len();
        for (n, (rule, decision)) in rules.into_iter().enumerate() {
            let content = match self.full {
                true => decision.content.as_str(),
                false => decision.brief(),
            };
            let (fork, below) = match n + 1 == count && notes.is_empty() {
                true => ("└─ ", "   "),
                false => ("├─ ", "│  "),
            };
            let first = format!("  {fork}§ {} ", short(&rule.id));
            let rest = format!("  {below}{}", " ".repeat("§ 12345678 ".chars().count()));
            self.text(lines, (first, rest), content);
        }
        self.under(notes, "", json, lines)
    }
    /// Prints `lines`, each row's details in columns at the right edge of the width, or after the widest
    /// title; in the short tree, a row's title is cut so it fits, and the start time goes when a title
    /// would get shorter than 20 characters.
    fn show(&self, lines: Vec<Line>, out: &mut dyn Write) -> Result<(), Failure> {
        let mut widths = [0; 4];
        for line in &lines {
            if let Line::Row { cells, .. } = line {
                for (width, cell) in widths.iter_mut().zip(cells) {
                    *width = (*width).max(cell.as_deref().map_or(0, visible));
                }
            }
        }
        if let (false, Some(width)) = (self.full, self.width) {
            let short = lines.iter().any(|l| match l {
                Line::Row { lead, title, .. } => {
                    width.saturating_sub(visible(lead) + room(&widths)) < visible(title).min(20)
                }
                Line::Text(_) => false,
            });
            if short {
                widths[3] = 0;
            }
        }
        let room = room(&widths);
        let span = room.saturating_sub(2);
        let edge = match (self.full, self.width) {
            (false, Some(width)) => width,
            (_, width) => lines
                .iter()
                .filter_map(|l| match l {
                    Line::Row { lead, title, .. } => Some(visible(lead) + visible(title) + room),
                    Line::Text(_) => None,
                })
                .chain(width)
                .max()
                .unwrap_or(0),
        };
        for line in lines {
            let (lead, title, cells) = match line {
                Line::Text(text) => {
                    super::put(out, &text)?;
                    continue;
                }
                Line::Row { lead, title, cells } => (lead, title, cells),
            };
            let title = match (self.full, self.width) {
                (false, Some(_)) => cut(&title, edge.saturating_sub(visible(&lead) + room).max(2)),
                _ => title,
            };
            let block: Vec<String> = cells
                .iter()
                .zip(widths)
                .enumerate()
                .filter(|(_, (_, width))| *width > 0)
                .map(|(n, (cell, width))| {
                    let cell = cell.as_deref().unwrap_or("");
                    let pad = " ".repeat(width - visible(cell));
                    match n {
                        0 | 1 => format!("{cell}{pad}"),
                        _ => format!("{pad}{cell}"),
                    }
                })
                .collect();
            let block = block.join("  ");
            let gap = edge.saturating_sub(visible(&lead) + visible(&title) + span);
            let line = match block.trim_end() {
                "" => format!("{lead}{title}"),
                block => format!("{lead}{title}{}{}", " ".repeat(gap), self.paint(DIM, block)),
            };
            super::put(out, &line)?;
        }
        Ok(())
    }

    /// Adds the steps `notes`, each followed by its own, after the branch lines `branch` of the steps
    /// above.
    fn under(&self, notes: Vec<Step>, branch: &str, json: bool, lines: &mut Vec<Line>) {
        let count = notes.len();
        for (n, note) in notes.into_iter().enumerate() {
            let (fork, below) = match n + 1 == count {
                true => ("└─ ", "   "),
                false => ("├─ ", "│  "),
            };
            let next = format!("{branch}{below}");
            let step = note.record();
            let done = note.done();
            let grade = note.grade();
            let reason = note.dropped().map(|d| d.content.clone());
            let state = note.state();
            if json {
                let mut value = super::json::step(note.note(), step);
                value["state"] = json!(state.text());
                value["proof"] = json!(done.map(|d| d.content.clone()));
                value["changed"] = json!(done.and_then(|d| d.tag("changed")));
                value["grade"] = json!(grade);
                value["reused"] = json!(self.reuse.down(note).len());
                value["reason"] = json!(reason);
                lines.push(Line::Text(json!({ "step": value }).to_string()));
            } else {
                if state.active() {
                    self.until(note.trail().newest() + LIFE);
                }
                let (mark, colour) = look(state, false);
                let lead = format!(
                    "{}{} {} ",
                    self.paint(DIM, &format!("  {branch}{fork}")),
                    self.paint(colour, mark),
                    self.id(note)
                );
                let start = note.start();
                let time = match (&grade, start) {
                    (Some(grade), _) => Some(took(grade.took)),
                    (None, Some(start)) if state == State::Working => {
                        Some(self.since(start.created_at))
                    }
                    _ => None,
                };
                let column = visible(&lead);
                lines.push(Line::Row {
                    lead,
                    title: step.title().to_string(),
                    cells: [
                        step.at().map(last),
                        None,
                        time,
                        start.map(|s| self.clock(s.created_at)),
                    ],
                });
                let rail = format!("  {next}");
                self.closing(step, done, &grade, &reason, (&rail, column), lines);
                self.reused(note, (&rail, column), lines);
            }
            self.under(note.children(), &next, json, lines);
        }
    }

    /// Adds under `step` the detail of its text, its proof, change and grade when `done`, or its reason when
    /// dropped, after the branch lines `at.0`, at the column `at.1` of the step's title.
    fn closing(
        &self,
        step: &StepRecord,
        done: Option<&celis_nostr::Note>,
        grade: &Option<Grade>,
        reason: &Option<String>,
        at: (&str, usize),
        out: &mut Vec<Line>,
    ) {
        if !self.full {
            return;
        }
        for line in step.detail() {
            self.labelled(out, at, "", line);
        }
        if let (Some(done), Some(grade)) = (done, grade) {
            self.labelled(out, at, "done", &done.content);
            if let Some(changed) = done.tag("changed") {
                self.labelled(out, at, "changed", changed);
            }
            let line = super::line::grade(grade);
            self.labelled(out, at, "took", line.trim_start_matches("took "));
        }
        if let Some(reason) = reason {
            self.labelled(out, at, "dropped", reason);
        }
    }

    /// Adds under `note` in the full tree a `follows` line for each step it follows, then a `reused` line
    /// when steps follow it, after the branch lines `at.0`, at the column `at.1`.
    fn reused(&self, note: Step, at: (&str, usize), out: &mut Vec<Line>) {
        if !self.full {
            return;
        }
        for source in self.reuse.sources(note) {
            let line = format!("{} {}", short(source.id()), source.title());
            self.labelled(out, at, "follows", &line);
        }
        let count = self.reuse.down(note).len();
        if count > 0 {
            let save = self.reuse.saved(note);
            let line = save.map_or(count.to_string(), |s| {
                format!("{count} · saved {}", saved(s))
            });
            self.labelled(out, at, "reused", &line);
        }
    }

    /// The first 8 characters of the id of `note`, its shortest start that no other step's id has
    /// underlined.
    fn id(&self, note: Step) -> String {
        let id = short(note.id());
        let all = self.steps.all();
        let unique = (4..id.len())
            .find(|&n| {
                all.iter()
                    .all(|other| other.id() == note.id() || !other.id().starts_with(&id[..n]))
            })
            .unwrap_or(id.len());
        format!(
            "{}{}",
            self.paint(UNDERLINE, &id[..unique]),
            self.paint(DIM, &id[unique..])
        )
    }

    /// Adds `text` dim after the branch lines of `at.0`, starting at the column `at.1`, with `label` ending
    /// one space before it.
    fn labelled(&self, lines: &mut Vec<Line>, at: (&str, usize), label: &str, text: &str) {
        let (rail, column) = at;
        let room = column.saturating_sub(visible(rail));
        let gap = match label {
            "" => room,
            _ => room.saturating_sub(label.len() + 1),
        };
        let first = match label {
            "" => format!("{rail}{}", " ".repeat(gap)),
            _ => format!("{rail}{}{label} ", " ".repeat(gap)),
        };
        let rest = format!("{rail}{}", " ".repeat(room));
        self.text(lines, (first, rest), text);
    }

    /// Adds `text` dim after `leads.0`, its next lines after `leads.1`. The full tree wraps it at the
    /// width, each line at the same column; the short tree cuts it.
    fn text(&self, lines: &mut Vec<Line>, leads: (String, String), text: &str) {
        let (first, rest) = leads;
        let column = visible(&first);
        let measure = self.width.map(|w| w.saturating_sub(column).max(20));
        let parts = match (self.full, measure) {
            (true, Some(measure)) => wrap(text, measure),
            (false, Some(measure)) => vec![cut(text, measure)],
            (_, None) => vec![text.to_string()],
        };
        for (n, part) in parts.iter().enumerate() {
            let lead = if n == 0 { &first } else { &rest };
            lines.push(Line::Text(self.paint(DIM, &format!("{lead}{part}"))));
        }
    }

    /// `took` of the time since `from`, which lowers `next` to the moment it changes.
    fn since(&self, from: u64) -> String {
        let seconds = self.steps.now.saturating_sub(from);
        self.until(self.steps.now + stride(seconds));
        took(seconds)
    }

    /// `clock` of `at`; a time of today lowers `next` to the midnight that turns it into a day.
    fn clock(&self, at: u64) -> String {
        let text = clock(at, self.steps.now);
        if text.contains(':') {
            self.until(midnight(self.steps.now) + 86_400);
        }
        text
    }

    /// Lowers `next` to `at`.
    fn until(&self, at: u64) {
        self.next
            .set(Some(self.next.get().map_or(at, |n| n.min(at))));
    }

    /// `text` in the colour `code` when in colour; else `text`.
    pub fn paint(&self, code: &str, text: &str) -> String {
        match self.colour {
            true => format!("\x1b[{code}m{text}\x1b[0m"),
            false => text.to_string(),
        }
    }
}

const DIM: &str = "2";
const BOLD: &str = "1";
const UNDERLINE: &str = "4;34";
/// The characters the detail columns of `widths` take with the two spaces before them; none when all are
/// empty.
fn room(widths: &[usize; 4]) -> usize {
    let shown = widths.iter().filter(|w| **w > 0).count();
    match shown {
        0 => 0,
        n => widths.iter().sum::<usize>() + 2 * n,
    }
}

/// The last part of the place `at`: a file's name with its line, a folder's name, or a URL's last part.
fn last(at: &str) -> String {
    let at = at.trim_end_matches('/');
    at.rsplit('/').next().unwrap_or(at).to_string()
}

/// The characters of `text` a terminal shows; colour codes count none.
fn visible(text: &str) -> usize {
    let mut count = 0;
    let mut code = false;
    for c in text.chars() {
        match (code, c) {
            (false, '\x1b') => code = true,
            (true, 'm') => code = false,
            (false, _) => count += 1,
            _ => {}
        }
    }
    count
}

/// `text` in lines of at most `width` characters, broken between words; a longer word stands alone.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = vec![];
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if visible(line) + 1 + visible(word) <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
}

/// `line` cut to `width` visible characters, its last one `…`, when it is wider.
fn cut(line: &str, width: usize) -> String {
    if width < 2 || visible(line) <= width {
        return line.to_string();
    }
    let mut kept = String::new();
    for c in line.chars() {
        kept.push(c);
        if visible(&kept) == width - 1 {
            break;
        }
    }
    kept.push('…');
    if line.contains('\x1b') {
        kept.push_str("\x1b[0m");
    }
    kept
}

/// The mark and colour of a step in `state`; an open or working trail is marked as active.
pub(super) fn look(state: State, trail: bool) -> (&'static str, &'static str) {
    match state {
        State::Open | State::Working if trail => ("●", "33"),
        State::Open => ("○", "39"),
        State::Working => ("◐", "33"),
        State::Done => ("✓", "32"),
        State::Revised => ("✓", "33"),
        State::Dropped => ("✗", "31"),
        State::Expired => ("!", "31"),
    }
}

#[cfg(test)]
mod tests;
