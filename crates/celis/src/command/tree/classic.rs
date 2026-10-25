//! `celis tree` as v0.0.11 printed and watched it: a trail with its state, its patterns and the steps under it.

use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::progress::{Flags, Grade, Model, Reuse, Tally};
pub(crate) use crate::terminal::canvas::visible;
use crate::terminal::canvas::{line as line_text, paint, width, Styled};
use crate::terminal::canvas::{BOLD, DIM};
use crate::terminal::classic::{look, signs, SPINNER};
use crate::terminal::look::SPUN;
use crate::text::clock::stride;
use crate::text::clock::{day, midnight};
use celis_trails::records::id::short;
use celis_trails::step::steps::LIFE;
use celis_trails::{Patterns, State, Step, Steps, Trail};
pub(crate) use layout::Layout;
use level::Level;
use ratatui::style::Style;
use ratatui::text::Span;
use row::Row;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::io::{IsTerminal, Write};

impl Command {
    /// Prints the step `args` names with every step under it, or with no id each trail of every project
    /// that is active or has a record from today, newest first, else `EMPTY`; on a terminal with no
    /// `--json`, again on each change. Writes no record.
    pub(crate) fn classic_tree(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let id = args.words.get(1);
        if std::io::stdout().is_terminal() && !args.json {
            return self.classic_watch(args, out);
        }
        let tree = match (self.held()?, id) {
            (true, _) => {
                let trails = self.trails()?;
                Model::of(&trails, |model| {
                    self.classic_frame(model, None, args, None, &View::of(args))
                })?
                .text
            }
            (false, Some(id)) => return Err(Failure::no_step(id)),
            (false, None) => Vec::new(),
        };
        if tree.is_empty() && id.is_none() && !args.all && !args.json {
            return crate::command::put(out, EMPTY);
        }
        for line in &tree {
            crate::command::put(out, &line_text(line, false))?;
        }
        Ok(())
    }

    /// The tree of `model` that `args` names, `width` wide, as `view` shows it, from the day groups
    /// `layout` kept.
    pub(crate) fn classic_frame(
        &self,
        model: &Model,
        layout: Option<&Layout>,
        args: &Args,
        width: Option<usize>,
        view: &View,
    ) -> Result<Frame, Failure> {
        let id = args.words.get(1);
        let steps = model.steps;
        let chosen: Vec<Step> = match (id, view.all) {
            (Some(id), _) => vec![steps.find(id)?],
            (None, true) => steps.roots().into_iter().rev().collect(),
            (None, false) => today(steps),
        };
        let tree = Tree {
            steps,
            patterns: model.patterns,
            reuse: model.reuse,
            layout,
            grouped: id.is_none(),
            width: width.map(|w| w.saturating_sub(2 * usize::from(view.watch))),
            view,
            spun: Cell::default(),
            order: RefCell::default(),
            printed: Cell::default(),
            picked: Cell::default(),
            next: Cell::default(),
        };
        let mut printed = Vec::new();
        tree.print(&chosen, "trail", args.json, &mut printed)?;
        Ok(tree.frame(printed))
    }
}

/// Each trail of every project that is active or has a record from the day of `steps`, newest first.
pub(crate) fn today<'a>(steps: &'a Steps<'a>) -> Vec<Step<'a>> {
    let midnight = midnight(steps.now());
    let roots = steps.roots().into_iter().rev();
    roots
        .filter(|s| s.state().active() || s.newest() >= midnight)
        .collect()
}

/// How a tree shows: in a watch, with a cursor column; full; all with no id; and in a watch, with the
/// spinner, the lines it opened or closed, and the line it selected.
#[derive(Default)]
pub(crate) struct View {
    pub watch: bool,
    pub full: bool,
    pub all: bool,
    pub spin: bool,
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

/// A printed tree or part of one: its lines, the earliest time a time it shows changes, whether it
/// shows the spinner, the lines a watch selects in the order it prints them, and the line of the
/// selected one.
pub(crate) struct Frame {
    pub text: Vec<Styled>,
    pub next: Option<u64>,
    pub spun: bool,
    pub order: Vec<Item>,
    pub picked: Option<usize>,
}

/// A trail, pattern or step a watch selects: its id, whether it is open, the trail or step it is under,
/// and every pattern and step under it.
#[derive(Clone)]
pub(crate) struct Item {
    pub id: String,
    pub open: bool,
    pub parent: Option<String>,
    pub under: Vec<String>,
}

impl Item {
    /// The first 8 characters of its id; none for a patterns or closed line.
    pub fn copy(&self) -> Option<String> {
        let group = self.id.starts_with(PATTERNS) || self.id.starts_with(CLOSED);
        (!group).then(|| short(&self.id))
    }
}

/// What `tree` with no id prints while it has no trail to print.
pub(crate) const EMPTY: &str = "No active trails. Run `celis tree --all` to see every trail.";

/// The steps and patterns a tree is printed from, the day groups it keeps, and how: grouped by
/// day and project; `width` wide; as `view` shows it, off a watch as the agent view. `next` holds the
/// earliest time a time it printed changes, `spun` whether it printed the spinner, `order` the lines a
/// watch selects, `printed` the count of its lines, and `picked` the line of the selected trail.
pub(crate) struct Tree<'a> {
    pub steps: &'a Steps<'a>,
    pub patterns: &'a Patterns<'a>,
    pub reuse: &'a Reuse<'a>,
    pub layout: Option<&'a Layout>,
    pub grouped: bool,
    pub width: Option<usize>,
    pub view: &'a View,
    pub spun: Cell<bool>,
    pub order: RefCell<Vec<Item>>,
    pub printed: Cell<usize>,
    pub picked: Cell<Option<usize>>,
    pub next: Cell<Option<u64>>,
}

/// A live pattern: its record and its pattern.
type PatternNote<'a> = (&'a celis_nostr::Note, celis_trails::Pattern);

/// What the tree says of a trail: the steps it prints under it, the done ones, the time they worked,
/// and its patterns.
struct Summary<'a> {
    all: usize,
    done: usize,
    worked: Option<u64>,
    patterns: Vec<PatternNote<'a>>,
}

impl Tree<'_> {
    /// Prints `notes` as the tree's mode gives them: grouped, each day's after a line with the day and its
    /// counts, by project, each project's active ones first, and in a watch a `Show more` line last;
    /// else each as a block. With `json`, each as its object named `key`, then its steps.
    pub fn print(
        &self,
        notes: &[Step],
        key: &str,
        json: bool,
        out: &mut Vec<Styled>,
    ) -> Result<(), Failure> {
        if json {
            let mut lines = vec![];
            for &note in notes {
                self.json(note, key, &mut lines);
            }
            return self.show(lines, out);
        }
        if !self.grouped {
            let (mut lines, mut last) = (vec![], None);
            for &note in notes {
                self.trail(note, true, &mut last, &mut lines);
            }
            return self.show(lines, out);
        }
        let mut days: Vec<(u64, String, Vec<Step>)> = vec![];
        for &note in notes {
            let at = match self.view.all {
                true => note.note().created_at,
                false => self.steps.now(),
            };
            let name = day(at);
            match days.last_mut() {
                Some((_, last, trails)) if *last == name => trails.push(note),
                _ => days.push((at, name, vec![note])),
            }
        }
        for (n, (at, name, trails)) in days.into_iter().enumerate() {
            if n > 0 {
                self.put(out, Styled::default())?;
            }
            self.group((at, name), trails, out)?;
        }
        if self.view.watch && !notes.is_empty() {
            self.put(out, Styled::default())?;
            let (chevron, label) = match self.view.all {
                true => ("▾", "Show less"),
                false => ("▸", "Show more"),
            };
            let mut lines = vec![Row::Titled {
                lead: Styled::default(),
                marks: Styled::from(paint(DIM, chevron)),
                title: label.to_string(),
                tone: Style::new(),
                rest: Styled::from("  "),
            }];
            let item = Item {
                id: MORE.to_string(),
                open: self.view.all,
                parent: None,
                under: vec![],
            };
            self.pick(item, &mut lines, 0);
            self.show(lines, out)?;
        }
        Ok(())
    }

    /// The step that holds the work of `note`: a trail's first ask with its text, else `note`.
    fn work<'b>(&self, note: Step<'b>) -> Step<'b> {
        match note.closed() {
            Some(_) => note,
            None => Trail::new(note).work(),
        }
    }

    /// What the tree says of the trail `note`, whose work `work` holds: the steps it prints under it, the
    /// done ones, the time they worked, and its patterns. An active one lowers `next` to its end.
    fn summary(&self, note: Step, work: Step) -> Summary<'_> {
        if note.state().active() {
            self.until(note.newest() + LIFE);
        }
        let parts = Trail::new(note).parts().into_iter();
        let shown: Vec<Step> = parts.flat_map(|p| p.tree()).collect();
        let worked = match shown.is_empty() {
            true => self.worked(&[work]),
            false => self.worked(&shown),
        };
        Summary {
            all: shown.len(),
            done: Tally::of(&shown).done,
            worked,
            patterns: self.patterns(note),
        }
    }

    /// The live patterns of the trail of `note`: its own, or its trail's when it is an ask.
    fn patterns(&self, note: Step) -> Vec<PatternNote<'_>> {
        let owner = match note.is_ask() {
            true => note.trail(),
            false => note,
        };
        self.patterns.of(owner)
    }

    /// The time the steps of `tree` with no step under them worked: what a done one took, and the time
    /// since a working one started; `None` when none worked. A working one lowers `next`.
    fn worked(&self, tree: &[Step]) -> Option<u64> {
        let now = self.steps.now();
        let leaves = tree.iter().filter(|s| s.children().is_empty());
        let (mut total, mut any, mut running) = (0, false, false);
        for leaf in leaves {
            match (Grade::of(*leaf), leaf.start()) {
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

    /// Prints `lines`, each row's marks padded to the widest of them, so every title starts at one
    /// column, and wrapped at the width. In a watch, each line starts with the cursor column, a bar on
    /// each line of the selected element.
    fn show(&self, lines: Vec<Row>, out: &mut Vec<Styled>) -> Result<(), Failure> {
        let slot = lines.iter().filter_map(Row::marks).map(width).max();
        let slot = slot.unwrap_or(0);
        let mut picked = false;
        for line in lines {
            let line = match line {
                Row::Mark(selected, line) => {
                    picked = selected;
                    if picked && self.picked.get().is_none() {
                        self.picked.set(Some(self.printed.get()));
                    }
                    *line
                }
                line => line,
            };
            for mut text in line.lines(slot, self.width, picked) {
                match (self.view.watch, picked) {
                    (false, _) => {}
                    (true, false) => text.spans.insert(0, Span::raw("  ")),
                    (true, true) => text.spans.insert(0, Span::raw("▌ ")),
                }
                self.put(out, text)?;
            }
        }
        Ok(())
    }

    /// Adds the line `text`, and counts it.
    fn put(&self, out: &mut Vec<Styled>, text: Styled) -> Result<(), Failure> {
        self.printed.set(self.printed.get() + 1);
        out.push(text);
        Ok(())
    }

    /// The mark of `note` with its flags after it, each in its colour.
    fn marked(&self, note: Step) -> Styled {
        let signs = signs(Flags::of(note)).into_iter();
        let flags = signs.map(|(sign, style)| paint(style, sign));
        std::iter::once(self.mark(note.state()))
            .chain(flags)
            .collect()
    }

    /// The mark of a step in `state` in its colour; in a watch, a working one's is the spinner.
    fn mark(&self, state: State) -> Span<'static> {
        let look = look(state);
        if state == State::Working && self.view.spin {
            self.spun.set(true);
            return paint(SPINNER, SPUN);
        }
        paint(look.text, look.mark)
    }

    /// The colour of a title usually in `usual`: bold for 2 seconds after its newest record at `newest`.
    fn tone(&self, usual: Style, newest: u64) -> Style {
        if newest + FRESH <= self.steps.now() {
            return usual;
        }
        self.until(newest + FRESH);
        BOLD
    }

    /// Lowers `next` to `at`.
    fn until(&self, at: u64) {
        self.next
            .set(Some(self.next.get().map_or(at, |n| n.min(at))));
    }
}

/// `rail` with `more` after it.
fn after(rail: &Styled, more: Span<'static>) -> Styled {
    let mut line = rail.clone();
    line.push_span(more);
    line
}

/// The seconds a step's title prints bold after its newest record.
const FRESH: u64 = 2;
/// What starts the id of a trail's patterns line in a watch's order.
const PATTERNS: &str = "§";
/// The id of the `Show more` line in a watch's order.
pub(crate) const MORE: &str = "+more";
/// What starts the id of a trail's closed line in a watch's order.
const CLOSED: &str = "✓";
/// The colour of a project's name.
const PROJECT: Style = BOLD;
mod agent;
mod day;
mod json;
mod layout;
mod level;
mod row;
#[cfg(test)]
mod tests;
mod trail;
mod watch;
