//! `celis tree`: a trail with its state, its rules and the steps under it.

use super::clock::{day, midnight};
use super::clock::{saved, stride, took};
use super::{Args, Command};
use celis_trails::id::short;
use celis_trails::step::steps::LIFE;
use celis_trails::{Decisions, Failure, Reuse, State, Step, Steps, Trail, Trails};
use serde_json::json;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
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
            (true, _) => {
                let view = View::of(args);
                self.frame(&self.trails()?, args, self.columns(), &view)?
                    .text
            }
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

    /// The tree of `trails` that `args` names, `width` wide, as `view` shows it.
    pub(super) fn frame(
        &self,
        trails: &Trails,
        args: &Args,
        width: Option<usize>,
        view: &View,
    ) -> Result<Frame, Failure> {
        let id = args.words.get(1);
        let all = view.all && id.is_none();
        let steps = trails.steps();
        let decisions = trails.decisions();
        let reuse = Reuse::new(&steps);
        let chosen: Vec<Step> = match (id, all) {
            (Some(id), _) => vec![steps.find(id)?],
            (None, true) => steps.roots().into_iter().rev().collect(),
            (None, false) => today(&steps, trails.now),
        };
        let tree = Tree {
            steps: &steps,
            decisions: &decisions,
            reuse: &reuse,
            colour: self.colour(),
            full: view.full,
            grouped: id.is_none() && !all,
            all,
            width,
            view,
            spun: Cell::default(),
            order: RefCell::default(),
            next: Cell::default(),
        };
        let mut printed = Vec::new();
        tree.print(&chosen, "trail", args.json, &mut printed)?;
        Ok(Frame {
            text: String::from_utf8_lossy(&printed).into_owned(),
            next: tree.next.get(),
            spun: tree.spun.get(),
            order: tree.order.take(),
        })
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

/// How a tree shows: full, all, and in a watch, the spinner's frame, the trails whose block or line it
/// flipped, and the trail it selected.
#[derive(Default)]
pub(super) struct View {
    pub full: bool,
    pub all: bool,
    pub spin: Option<usize>,
    pub flipped: HashSet<String>,
    pub selected: Option<String>,
}

impl View {
    /// The view the flags of `args` name.
    pub fn of(args: &Args) -> View {
        View {
            full: args.full,
            all: args.all,
            ..View::default()
        }
    }
}

/// A printed tree: its text, the earliest time a time it shows changes, whether it shows the spinner,
/// and the ids of its trails in the order it prints them.
pub(super) struct Frame {
    pub text: String,
    pub next: Option<u64>,
    pub spun: bool,
    pub order: Vec<String>,
}

/// What `tree` with no id prints while it has no trail to print.
pub(super) const EMPTY: &str = "No active trails. Run `celis tree --all` to see every trail.";

/// The steps and decisions a tree is printed from, and how: in colour; full, with every trail as a
/// block and the lines under each step; grouped, as today's `ACTIVE` and `DONE` trails; all, by day;
/// else each line cut to `width`; as `view` shows it. `next` holds the earliest time a time it printed
/// changes, `spun` whether it printed the spinner, and `order` the trails it printed.
pub(super) struct Tree<'a> {
    pub steps: &'a Steps<'a>,
    pub decisions: &'a Decisions<'a>,
    pub reuse: &'a Reuse<'a>,
    pub colour: bool,
    pub full: bool,
    pub grouped: bool,
    pub all: bool,
    pub width: Option<usize>,
    pub view: &'a View,
    pub spun: Cell<bool>,
    pub order: RefCell<Vec<String>>,
    pub next: Cell<Option<u64>>,
}

/// A line of the tree: text as it is, or a row whose details line up with the other rows' of its group.
enum Line {
    Text(String),
    Row {
        lead: String,
        title: String,
        cells: [Option<String>; 4],
    },
}

impl Tree<'_> {
    /// Prints `notes` as the tree's mode gives them: grouped, the active ones as blocks after `ACTIVE` and
    /// the others after `DONE`; with `all`, each day's after a line with the day; else each as a block.
    /// With `json`, each as its object named `key`, then its steps.
    pub fn print(
        &self,
        notes: &[Step],
        key: &str,
        json: bool,
        out: &mut dyn Write,
    ) -> Result<(), Failure> {
        if json {
            let mut lines = vec![];
            for &note in notes {
                self.json(note, key, &mut lines);
            }
            return self.show(lines, out);
        }
        let mut groups: Vec<Vec<Line>> = vec![];
        if self.grouped {
            let (active, done): (Vec<Step>, Vec<Step>) =
                notes.iter().partition(|s| s.state().active());
            for (name, trails) in [("ACTIVE", active), ("DONE", done)] {
                if trails.is_empty() {
                    continue;
                }
                let mut lines = vec![Line::Text(self.paint(BOLD, name))];
                let mut last = None;
                for &note in &trails {
                    let block = self.full || name == "ACTIVE";
                    self.trail(note, block, &mut last, &mut lines);
                }
                groups.push(lines);
            }
        } else {
            let mut lines = vec![];
            let (mut today, mut last) = (None, None);
            for &note in notes {
                let day = day(note.note().created_at);
                if self.all && today.as_ref() != Some(&day) {
                    lines.push(Line::Text(self.paint(BOLD, &day)));
                    (today, last) = (Some(day), None);
                }
                let block = !self.all || self.full;
                self.trail(note, block, &mut last, &mut lines);
            }
            groups.push(lines);
        }
        for (n, group) in groups.into_iter().enumerate() {
            if n > 0 {
                super::put(out, "")?;
            }
            self.show(group, out)?;
        }
        Ok(())
    }

    /// Adds the trail `note` as a block when `block`, else as one line, the other way when the view
    /// flipped it; a blank line comes between it and a `last` trail when either is a block.
    fn trail(&self, note: Step, block: bool, last: &mut Option<bool>, lines: &mut Vec<Line>) {
        let block = block != self.view.flipped.contains(note.id());
        if last.is_some_and(|last| last || block) {
            lines.push(Line::Text(String::new()));
        }
        self.order.borrow_mut().push(note.id().to_string());
        match block {
            true => self.block(note, lines),
            false => self.line(note, lines),
        }
        *last = Some(block);
    }

    /// Adds the object of the step `note`, named `key`, then each step under it as its own.
    fn json(&self, note: Step, key: &str, lines: &mut Vec<Line>) {
        let step = note.record();
        let work = self.work(note);
        let done = work.done();
        let head = json!({"id": note.id(), "created_at": note.note().created_at,
            "project": step.anchor.project, "content": step.content, "state": note.state().text(),
            "links": super::json::links(step), "places": step.places, "session": step.session,
            "proof": done.map(|d| d.content.clone()),
            "changed": done.and_then(|d| d.tag("changed")),
            "grade": work.grade(),
            "reused": self.reuse.down(work).len(),
            "reason": work.dropped().map(|d| d.content.clone())});
        lines.push(Line::Text(json!({ key: head }).to_string()));
        self.objects(note.children(), lines);
    }

    /// Adds the object of each step of `notes`, each followed by its own.
    fn objects(&self, notes: Vec<Step>, lines: &mut Vec<Line>) {
        for note in notes {
            let step = note.record();
            let done = note.done();
            let mut value = super::json::step(note.note(), step);
            value["state"] = json!(note.state().text());
            value["proof"] = json!(done.map(|d| d.content.clone()));
            value["changed"] = json!(done.and_then(|d| d.tag("changed")));
            value["grade"] = json!(note.grade());
            value["reused"] = json!(self.reuse.down(note).len());
            value["reason"] = json!(note.dropped().map(|d| d.content.clone()));
            lines.push(Line::Text(json!({ "step": value }).to_string()));
            self.objects(note.children(), lines);
        }
    }

    /// The step that holds the work of `note`: a trail's first ask with its text, else `note`.
    fn work<'b>(&self, note: Step<'b>) -> Step<'b> {
        match note.closed() {
            Some(_) => note,
            None => Trail::new(note).work(),
        }
    }

    /// The steps of the tree of `note` that the tree prints: all under it, but a trail's first ask with
    /// its text.
    fn shown<'b>(&self, note: Step<'b>) -> Vec<Step<'b>> {
        let twin = Trail::new(note).first().map(|s| s.id());
        let tree = note.tree().into_iter().skip(1);
        tree.filter(|s| Some(s.id()) != twin).collect()
    }

    /// The live rules of the trail of `note`: its own, or its trail's when it is an ask.
    fn rules(&self, note: Step) -> Vec<(&celis_nostr::Note, celis_trails::Decision)> {
        let owner = match note.is_ask() {
            true => note.trail(),
            false => note,
        };
        self.decisions
            .rules(&note.record().anchor.project, owner.id())
    }

    /// Adds `note` as a block: its mark and title, its project, progress, worked time, rules and id, the
    /// lines under it and its rules with `full`, then its steps.
    fn block(&self, note: Step, lines: &mut Vec<Line>) {
        let step = note.record();
        let state = note.state();
        if state.active() {
            self.until(note.newest() + LIFE);
        }
        let width = self.width.map(|w| w.saturating_sub(2).max(20));
        let title = match width {
            Some(width) => wrap(step.title(), width),
            None => vec![step.title().to_string()],
        };
        for (n, part) in title.iter().enumerate() {
            let lead = match n {
                0 => format!("{} ", self.mark(state)),
                _ => "  ".to_string(),
            };
            let style = match self.picked(note) {
                true => PICKED,
                false => BOLD,
            };
            lines.push(Line::Text(format!("{lead}{}", self.paint(style, part))));
        }
        let shown = self.shown(note);
        let rules = self.rules(note);
        let mut parts = vec![step.anchor.name.clone()];
        if !shown.is_empty() {
            let done = shown
                .iter()
                .filter(|s| matches!(s.state(), State::Done | State::Revised))
                .count();
            parts.push(format!("{done} of {}", shown.len()));
        }
        let leaves = match shown.is_empty() {
            true => vec![self.work(note)],
            false => shown.clone(),
        };
        parts.extend(self.worked(&leaves).map(took));
        if !rules.is_empty() {
            parts.push(format!("§ {}", count(rules.len(), "rule")));
        }
        if self.view.spin.is_some() && state.active() {
            let seconds = self.steps.now.saturating_sub(note.newest());
            self.until(self.steps.now + stride(seconds));
            parts.push(format!("active {} ago", took(seconds)));
        }
        lines.push(Line::Row {
            lead: "  ".to_string(),
            title: self.paint(DIM, &parts.join(" · ")),
            cells: [None, None, None, Some(self.id(note))],
        });
        let work = self.work(note);
        self.closing(work, "  ", lines);
        self.reused(work, "  ", lines);
        if self.full {
            for (rule, decision) in rules {
                let first = format!("  § {} ", short(&rule.id));
                let rest = " ".repeat(visible(&first));
                self.text(lines, (first, rest), &decision.content);
            }
        }
        let parts = Trail::new(note).parts();
        if !parts.is_empty() {
            lines.push(Line::Text(String::new()));
        }
        self.under(parts, 0, lines);
    }

    /// Adds `note` as one line: its mark and title, then its project, steps, rules, worked time and id;
    /// a dropped one's reason on the next line.
    fn line(&self, note: Step, lines: &mut Vec<Line>) {
        let step = note.record();
        let state = note.state();
        if state.active() {
            self.until(note.newest() + LIFE);
        }
        let shown = self.shown(note);
        let done = shown
            .iter()
            .filter(|s| matches!(s.state(), State::Done | State::Revised))
            .count();
        let mut summary = match (shown.len(), done) {
            (0, _) => vec![],
            (all, done) if all == done => vec![count(all, "step")],
            (all, done) => vec![format!("{done} of {all}")],
        };
        let rules = self.rules(note).len();
        if rules > 0 {
            summary.push(count(rules, "rule"));
        }
        let leaves = match shown.is_empty() {
            true => vec![self.work(note)],
            false => shown,
        };
        lines.push(Line::Row {
            lead: format!("{} ", self.mark(state)),
            title: match self.picked(note) {
                true => self.paint(PICKED, step.title()),
                false => self.title(state, step.title(), note.newest()),
            },
            cells: [
                Some(step.anchor.name.clone()),
                (!summary.is_empty()).then(|| summary.join(" · ")),
                self.worked(&leaves).map(took),
                Some(self.id(note)),
            ],
        });
        if let Some(reason) = self.work(note).dropped() {
            self.text(lines, ("  ".into(), "  ".into()), &reason.content);
        }
    }

    /// Adds the steps `notes`, each followed by its own, `depth` levels under a block's top steps.
    fn under(&self, notes: Vec<Step>, depth: usize, lines: &mut Vec<Line>) {
        for note in notes {
            let step = note.record();
            let state = note.state();
            if state.active() {
                self.until(note.trail().newest() + LIFE);
            }
            let guides = self.paint(DIM, &"│ ".repeat(depth));
            let marks = [note.start(), note.closed()].into_iter().flatten();
            let changed = marks.map(|n| n.created_at).max();
            let changed = changed.unwrap_or(note.note().created_at);
            lines.push(Line::Row {
                lead: format!("  {guides}{} ", self.mark(state)),
                title: self.title(state, step.title(), changed),
                cells: [
                    step.at().map(last),
                    None,
                    self.worked(&note.tree()).map(took),
                    Some(self.id(note)),
                ],
            });
            let rail = format!("  {guides}  ");
            if let (false, Some(changed)) = (self.full, note.done().and_then(|d| d.tag("changed")))
            {
                let first = format!("{rail}~ ");
                self.text(lines, (first, rail.clone()), changed);
            }
            self.closing(note, &rail, lines);
            self.reused(note, &rail, lines);
            self.under(note.children(), depth + 1, lines);
        }
    }

    /// The time the steps of `tree` with no step under them worked: what a done one took, and the time
    /// since a working one started; `None` when none worked. A working one lowers `next`.
    fn worked(&self, tree: &[Step]) -> Option<u64> {
        let now = self.steps.now;
        let leaves = tree.iter().filter(|s| s.children().is_empty());
        let (mut total, mut any, mut running) = (0, false, false);
        for leaf in leaves {
            match (leaf.grade(), leaf.start()) {
                (Some(grade), _) => total += grade.took,
                (None, Some(start)) if leaf.state() == State::Working => {
                    total += now.saturating_sub(start.created_at);
                    running = true;
                }
                _ => continue,
            }
            any = true;
        }
        if running {
            self.until(now + stride(total));
        }
        any.then_some(total)
    }

    /// Prints `lines`, each row's details in columns at the right edge of the width, or after the widest
    /// title; in the short tree, a row's title is cut so it fits, and a narrow width leaves out the ids,
    /// then the first column and the time while a title would be cut below 20 characters.
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
            if width < 80 {
                widths[3] = 0;
            }
            for column in [0, 2] {
                let short = lines.iter().any(|l| match l {
                    Line::Row { lead, title, .. } => {
                        width.saturating_sub(visible(lead) + room(&widths)) < visible(title).min(20)
                    }
                    Line::Text(_) => false,
                });
                if short {
                    widths[column] = 0;
                }
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
                        2 => format!("{pad}{cell}"),
                        _ => format!("{cell}{pad}"),
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

    /// Adds under `note` with `full` the lines of its text, its proof, change and grade when done, or its
    /// reason when dropped, after `rail`.
    fn closing(&self, note: Step, rail: &str, out: &mut Vec<Line>) {
        if !self.full {
            return;
        }
        for line in note.record().detail() {
            self.labelled(out, rail, "", line);
        }
        if let (Some(done), Some(grade)) = (note.done(), note.grade()) {
            self.labelled(out, rail, "done", &done.content);
            if let Some(changed) = done.tag("changed") {
                self.labelled(out, rail, "changed", changed);
            }
            let line = super::line::grade(&grade);
            self.labelled(out, rail, "took", line.trim_start_matches("took "));
        }
        if let Some(reason) = note.dropped() {
            self.labelled(out, rail, "dropped", &reason.content);
        }
    }

    /// Adds under `note` with `full` a `follows` line for each step it follows, then a `reused` line when
    /// steps follow it, after `rail`.
    fn reused(&self, note: Step, rail: &str, out: &mut Vec<Line>) {
        if !self.full {
            return;
        }
        for source in self.reuse.sources(note) {
            let line = format!("{} {}", short(source.id()), source.title());
            self.labelled(out, rail, "follows", &line);
        }
        let count = self.reuse.down(note).len();
        if count > 0 {
            let save = self.reuse.saved(note);
            let line = save.map_or(count.to_string(), |s| {
                format!("{count} · saved {}", saved(s))
            });
            self.labelled(out, rail, "reused", &line);
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

    /// Adds `text` dim after `rail` and `label` padded to 8 characters, or after `rail` alone with no
    /// label.
    fn labelled(&self, lines: &mut Vec<Line>, rail: &str, label: &str, text: &str) {
        let (first, rest) = match label {
            "" => (rail.to_string(), rail.to_string()),
            _ => (
                format!("{rail}{label:<8}"),
                format!("{rail}{}", " ".repeat(8)),
            ),
        };
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

    /// The mark of a step in `state` in its colour; in a watch, a working one's is the spinner.
    fn mark(&self, state: State) -> String {
        let look = look(state);
        let mark = match (state, self.view.spin) {
            (State::Working, Some(frame)) => {
                self.spun.set(true);
                SPIN[frame % SPIN.len()]
            }
            _ => look.mark,
        };
        self.paint(look.text, mark)
    }

    /// The title `text` of a step in `state` in its colour, bold for 2 seconds after its record at
    /// `changed`.
    fn title(&self, state: State, text: &str, changed: u64) -> String {
        if changed + FRESH <= self.steps.now {
            return self.paint(look(state).title, text);
        }
        self.until(changed + FRESH);
        self.paint(BOLD, text)
    }

    /// Whether the view selected the trail `note`.
    fn picked(&self, note: Step) -> bool {
        self.view.selected.as_deref() == Some(note.id())
    }

    /// Lowers `next` to `at`.
    fn until(&self, at: u64) {
        self.next
            .set(Some(self.next.get().map_or(at, |n| n.min(at))));
    }

    /// `text` in the colour `code` when in colour; else `text`.
    pub fn paint(&self, code: &str, text: &str) -> String {
        match self.colour && !code.is_empty() {
            true => format!("\x1b[{code}m{text}\x1b[0m"),
            false => text.to_string(),
        }
    }
}

const DIM: &str = "2";
/// The seconds a step's title prints bold after its newest record.
const FRESH: u64 = 2;
/// The frames of the spinner a watch draws for a working step.
const SPIN: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const BOLD: &str = "1";
const PICKED: &str = "7";
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
pub(super) fn cut(line: &str, width: usize) -> String {
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

/// How a step in a state prints: its mark, the colour of its mark, and the colour of its title.
pub(super) struct Look {
    pub mark: &'static str,
    pub text: &'static str,
    pub title: &'static str,
}

/// How a step or a trail in `state` prints.
pub(super) fn look(state: State) -> Look {
    let (mark, text, title) = match state {
        State::Open => ("○", "", ""),
        State::Working => ("◐", "33", BOLD),
        State::Done => ("✓", DIM, DIM),
        State::Revised => ("↻", DIM, DIM),
        State::Dropped => ("✗", "31", DIM),
        State::Expired => ("!", "31", DIM),
    };
    Look { mark, text, title }
}

/// `n` of `what`, as `1 step` or `3 steps`.
fn count(n: usize, what: &str) -> String {
    match n {
        1 => format!("1 {what}"),
        n => format!("{n} {what}s"),
    }
}

#[cfg(test)]
mod tests;
