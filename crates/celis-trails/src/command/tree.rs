//! `celis tree`: a trail with its state, its rules and the steps under it.

use super::{Args, Command, Snapshot};
use crate::grade::took;
use crate::id::short;
use crate::{Decisions, Failure, Grade, Project, State, Status, Step, Steps};
use celis_wire::Note;
use serde_json::json;
use std::io::Write;

impl Command {
    /// Prints the step `args` names with every step under it, or with no id each trail of `project` that
    /// is active or has a record from today, newest first. Writes nothing.
    pub(super) fn tree(
        &self,
        args: &Args,
        out: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        let id = args.words.get(1);
        if args.all && id.is_some() {
            return Err(Failure::usage("tree takes an id or --all, not both"));
        }
        if !self.held()? {
            return id.map_or(Ok(()), |id| Err(Failure::no_step(id)));
        }
        let store = self.store()?;
        let snapshot = Snapshot::of(&store);
        let steps = snapshot.steps();
        let decisions = snapshot.decisions();
        let today = midnight(snapshot.now);
        let chosen: Vec<&Note> = match (id, args.all) {
            (Some(id), _) => vec![steps.find(id)?],
            (None, true) => steps.roots().into_iter().rev().collect(),
            (None, false) => steps
                .trails(&project.id)
                .into_iter()
                .rev()
                .filter(|n| steps.state(n) == State::Active || steps.newest(n) >= today)
                .collect(),
        };
        let tree = Tree {
            steps: &steps,
            decisions: &decisions,
            styled: self.styled(),
            full: args.full,
            fold: id.is_none() && !args.full,
            all: args.all,
            width: self.columns(),
        };
        if tree.styled && id.is_none() && !args.all && !args.json && !chosen.is_empty() {
            super::put(out, &tree.paint(BOLD, &project.name))?;
        }
        tree.print(&chosen, "trail", args.json, out)
    }
}

/// The steps and decisions a tree is printed from, and how: styled, in colour with branch lines and dim
/// details in columns; full, with the lines under each step; fold, with each trail that is not active as
/// one line; all, by day with each trail's project; else each line cut to `width`.
pub(super) struct Tree<'a> {
    pub steps: &'a Steps<'a>,
    pub decisions: &'a Decisions<'a>,
    pub styled: bool,
    pub full: bool,
    pub fold: bool,
    pub all: bool,
    pub width: Option<usize>,
}

/// A line of the tree: text as it is, or a styled row whose details line up with the other rows'.
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
        notes: &[&Note],
        key: &str,
        json: bool,
        out: &mut dyn Write,
    ) -> Result<(), Failure> {
        let mut lines = vec![];
        let mut last = None;
        for note in notes {
            let day = day(note.created_at);
            if self.all && !json && last.as_ref() != Some(&day) {
                lines.push(Line::Text(self.paint(BOLD, &day)));
                last = Some(day);
            }
            self.trail(note, key, json, &mut lines);
        }
        self.show(lines, out)
    }

    /// Adds the step `note` with its state, its proof and grade when done, and its rules, then the steps
    /// under it; styled, a first ask with its text stands on its line. With `json`, its object is `key`.
    fn trail(&self, note: &Note, key: &str, json: bool, lines: &mut Vec<Line>) {
        let Some(step) = self.steps.step(note) else {
            return;
        };
        let state = self.steps.state(note);
        let done = self.steps.done(&note.id);
        let proof = done.map(|d| d.content.clone());
        let grade = self.steps.grade(note);
        let reason = self.steps.dropped(&note.id).map(|d| d.content.clone());
        if json {
            let head = json!({"id": note.id, "created_at": note.created_at,
                "project": step.anchor.project, "content": step.content, "state": state.text(),
                "links": step.links(), "places": step.places, "session": step.session, "proof": proof,
                "grade": grade.as_ref().map(|g| g.value()),
                "reason": reason});
            lines.push(Line::Text(json!({ key: head }).to_string()));
            return self.under(self.steps.under(&note.id), 0, "", json, lines);
        }
        let folded = self.fold && state != State::Active;
        let count = self.steps.tree(note).len() - 1;
        let steps = match (folded, count) {
            (false, _) | (true, 0) => None,
            (true, 1) => Some("1 step".to_string()),
            (true, n) => Some(format!("{n} steps")),
        };
        let line = match self.styled {
            true => {
                let (mark, colour) = trail_look(state);
                let project = match self.all {
                    true => format!("{} ", self.paint(DIM, &step.anchor.name)),
                    false => String::new(),
                };
                let lead = format!("{} {} {project}", self.paint(colour, mark), self.id(note));
                let ran = match state {
                    State::Active => self.steps.now,
                    _ => self.steps.newest(note),
                };
                let note_cell: Vec<String> = [
                    (state != State::Active && state != State::ReachedAlone)
                        .then(|| state.text().to_string()),
                    steps,
                ]
                .into_iter()
                .flatten()
                .collect();
                Line::Row {
                    lead,
                    title: self.paint(BOLD, step.title()),
                    cells: [
                        step.at().map(last),
                        (!note_cell.is_empty()).then(|| note_cell.join(" · ")),
                        Some(took(ran.saturating_sub(note.created_at))),
                        Some(clock(note.created_at, self.steps.now)),
                    ],
                }
            }
            false => {
                let mut tail = format!(" · {}", state.text());
                if let Some(steps) = steps {
                    tail.push_str(&format!(" · {steps}"));
                }
                Line::Text(self.line(&step, note, "", &tail))
            }
        };
        lines.push(line);
        self.closing(&step, done, &grade, &reason, "", lines);
        if folded {
            return;
        }
        for (rule, decision) in self.decisions.rules(&step.anchor.project, &note.id) {
            self.put(lines, &decision.rule(rule));
        }
        let mut notes = self.steps.under(&note.id);
        let twin = notes
            .first()
            .and_then(|n| self.steps.step(n))
            .is_some_and(|ask| self.styled && ask.content == step.content);
        if twin {
            let ask = notes.remove(0);
            notes.splice(0..0, self.steps.under(&ask.id));
        }
        self.under(notes, 0, "", json, lines)
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

    /// Adds the steps `notes` at `depth`, each followed by its own; styled, after the branch lines
    /// `branch` of the steps above.
    fn under(
        &self,
        notes: Vec<&Note>,
        depth: usize,
        branch: &str,
        json: bool,
        lines: &mut Vec<Line>,
    ) {
        let count = notes.len();
        for (n, note) in notes.into_iter().enumerate() {
            let (fork, below) = match n + 1 == count {
                true => ("└─ ", "   "),
                false => ("├─ ", "│  "),
            };
            let next = format!("{branch}{below}");
            let Some(step) = self.steps.step(note) else {
                continue;
            };
            let done = self.steps.done(&note.id);
            let proof = done.map(|d| d.content.clone());
            let grade = self.steps.grade(note);
            let reason = self.steps.dropped(&note.id).map(|d| d.content.clone());
            let status = match self.steps.status(&note.id) {
                Status::Working | Status::Open if self.steps.ended(self.steps.trail(note)) => {
                    Status::Expired
                }
                status => status,
            };
            if json {
                let mut value = step.value(note);
                value["state"] = json!(status.text());
                value["proof"] = json!(proof);
                value["grade"] = json!(grade.as_ref().map(|g| g.value()));
                value["reason"] = json!(reason);
                lines.push(Line::Text(json!({ "step": value }).to_string()));
            } else {
                let (mark, colour) = step_look(status);
                let (line, pad) = match self.styled {
                    true => {
                        let lead = format!(
                            "{}{} {} ",
                            self.paint(DIM, &format!("  {branch}{fork}")),
                            self.paint(colour, mark),
                            self.id(note)
                        );
                        let start = self.steps.started(&note.id);
                        let time = match (&grade, start) {
                            (Some(grade), _) => Some(took(grade.took)),
                            (None, Some(start)) if status == Status::Working => {
                                Some(took(self.steps.now.saturating_sub(start.created_at)))
                            }
                            _ => None,
                        };
                        let row = Line::Row {
                            lead,
                            title: step.title().to_string(),
                            cells: [
                                step.at().map(last),
                                step.fixes
                                    .as_deref()
                                    .map(|f| format!("↳ fixes {}", short(f))),
                                time,
                                start.map(|s| clock(s.created_at, self.steps.now)),
                            ],
                        };
                        (row, format!("  {next}"))
                    }
                    false => {
                        let pad = "  ".repeat(depth);
                        let line = self.line(&step, note, &format!("{pad}{mark} "), "");
                        (Line::Text(line), pad)
                    }
                };
                lines.push(line);
                self.closing(&step, done, &grade, &reason, &pad, lines);
            }
            self.under(self.steps.under(&note.id), depth + 1, &next, json, lines);
        }
    }

    /// Adds under `step`, after `pad`, the detail of its text, its proof and grade when
    /// `done`, or its reason when dropped.
    fn closing(
        &self,
        step: &Step,
        done: Option<&Note>,
        grade: &Option<Grade>,
        reason: &Option<String>,
        pad: &str,
        out: &mut Vec<Line>,
    ) {
        if !self.full {
            return;
        }
        let mut lines: Vec<String> = step.detail().map(|l| format!("{pad}    {l}")).collect();
        if let (Some(done), Some(grade)) = (done, grade) {
            lines.push(format!("{pad}    done {}", done.content));
            lines.push(format!("{pad}    {}", grade.line()));
        }
        if let Some(reason) = reason {
            lines.push(format!("{pad}    dropped {reason}"));
        }
        out.extend(lines.iter().map(|l| Line::Text(self.paint(DIM, l))));
    }

    /// The plain line of the step `note` between `lead` and `tail`; in the short tree, its title is cut so
    /// the line fits the width.
    fn line(&self, step: &Step, note: &Note, lead: &str, tail: &str) -> String {
        let head = format!("{lead}{} [{}] ", short(&note.id), step.anchor.name);
        let place = step.at().map_or(String::new(), |at| format!(" · {at}"));
        let tail = format!("{place}{tail}");
        let text = match (self.full, self.width) {
            (false, Some(width)) => {
                let room = width.saturating_sub(visible(&head) + visible(&tail));
                cut(step.title(), room.max(2))
            }
            _ => step.title().to_string(),
        };
        format!("{head}{text}{tail}")
    }

    /// The first 8 characters of the id of `note`, its shortest start that no other step's id has
    /// underlined.
    fn id(&self, note: &Note) -> String {
        let id = short(&note.id);
        let unique = (4..id.len())
            .find(|&n| {
                self.steps
                    .all()
                    .iter()
                    .all(|other| other.id == note.id || !other.id.starts_with(&id[..n]))
            })
            .unwrap_or(id.len());
        format!(
            "{}{}",
            self.paint(UNDERLINE, &id[..unique]),
            self.paint(DIM, &id[unique..])
        )
    }

    /// Adds `line`, dim when styled, cut to the width in the short tree.
    fn put(&self, lines: &mut Vec<Line>, line: &str) {
        let line = match (self.full, self.width) {
            (false, Some(width)) => cut(line, width),
            _ => line.to_string(),
        };
        lines.push(Line::Text(self.paint(DIM, &line)));
    }

    /// `text` in the colour `code` when styled; else `text`.
    fn paint(&self, code: &str, text: &str) -> String {
        match self.styled {
            true => format!("\x1b[{code}m{text}\x1b[0m"),
            false => text.to_string(),
        }
    }
}

const DIM: &str = "2";
const BOLD: &str = "1";
const UNDERLINE: &str = "4;34";
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

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

/// The time `at` as `14:02` on the local day of `now`, else as `Oct 9`.
fn clock(at: u64, now: u64) -> String {
    let (then, today) = (local(at), local(now));
    match (then.tm_year, then.tm_yday) == (today.tm_year, today.tm_yday) {
        true => format!("{:02}:{:02}", then.tm_hour, then.tm_min),
        false => format!(
            "{} {}",
            MONTHS[then.tm_mon.clamp(0, 11) as usize],
            then.tm_mday
        ),
    }
}

/// The local day of `at`, as `Oct 9`.
fn day(at: u64) -> String {
    let tm = local(at);
    format!("{} {}", MONTHS[tm.tm_mon.clamp(0, 11) as usize], tm.tm_mday)
}

/// The time of the local midnight that starts the day of `now`.
fn midnight(now: u64) -> u64 {
    let tm = local(now);
    let since = tm.tm_hour * 3_600 + tm.tm_min * 60 + tm.tm_sec;
    now.saturating_sub(u64::try_from(since).unwrap_or(0))
}

/// The local calendar time of `at`.
fn local(at: u64) -> libc::tm {
    let time = libc::time_t::try_from(at).unwrap_or(0);
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    unsafe { libc::localtime_r(&time, &mut tm) };
    tm
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

/// The mark and colour of a trail in `state`.
fn trail_look(state: State) -> (&'static str, &'static str) {
    match state {
        State::Active => ("●", "33"),
        State::ReachedAlone => ("✓", "32"),
        State::ReachedWithHelp => ("✓", "36"),
        State::Dropped => ("✗", "31"),
        State::Expired => ("!", "31"),
    }
}

/// The mark and colour of a step in `status`.
fn step_look(status: Status) -> (&'static str, &'static str) {
    match status {
        Status::Open => ("○", "39"),
        Status::Working => ("◐", "33"),
        Status::Done => ("✓", "32"),
        Status::Dropped => ("✗", "31"),
        Status::Expired => ("!", "31"),
    }
}
