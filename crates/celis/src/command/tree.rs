//! `celis tree`: a trail with its state, its rules and the steps under it.

use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::progress::{Flags, Grade, Model, Reuse, Tally};
pub(crate) use crate::terminal::canvas::{cut, visible};
use crate::terminal::canvas::{line as line_text, paint, width, Styled};
use crate::terminal::canvas::{BOLD, DIM, MAGENTA};
use crate::terminal::look::{look, signs, SPINNER, SPUN};
use crate::text::clock::{day, midnight, weekday};
use crate::text::clock::{stride, took};
use crate::text::line::plural;
use celis_trails::records::id::short;
use celis_trails::step::steps::LIFE;
use celis_trails::{Decisions, State, Step, Steps, Trail};
pub(crate) use layout::Layout;
use layout::{Group, Key};
use level::{ids, Level};
use ratatui::style::Style;
use ratatui::text::Span;
pub(crate) use row::wrap;
use row::Row;
use serde_json::json;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::io::{IsTerminal, Write};

impl Command {
    /// Prints the step `args` names with every step under it, or with no id each trail of every project
    /// that is active or has a record from today, newest first, else `EMPTY`; on a terminal with no
    /// `--json`, again on each change. Writes no record.
    pub(crate) fn tree(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let id = args.words.get(1);
        if args.all && id.is_some() {
            return Err(Failure::usage("tree takes an id or --all, not both"));
        }
        if args.demo && id.is_some() {
            return Err(Failure::usage("tree takes an id or --demo, not both"));
        }
        if args.demo {
            return self.demo(args, out);
        }
        if args.graph {
            return self.graph(args, out);
        }
        if std::io::stdout().is_terminal() && !args.json {
            return self.watch(args, out);
        }
        let tree = match (self.held()?, id) {
            (true, _) => {
                let trails = self.trails()?;
                Model::of(&trails, |model| {
                    self.frame(model, None, args, None, &View::of(args))
                })?
                .text
            }
            (false, Some(id)) => return Err(Failure::no_step(id)),
            (false, None) => Vec::new(),
        };
        if tree.is_empty() && id.is_none() && !args.all && !args.json {
            return super::put(out, EMPTY);
        }
        for line in &tree {
            super::put(out, &line_text(line, false))?;
        }
        Ok(())
    }

    /// The tree of `model` that `args` names, `width` wide, as `view` shows it, from the day groups
    /// `layout` kept.
    pub(crate) fn frame(
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
            decisions: model.decisions,
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

/// A trail, rule or step a watch selects: its id, whether it is open, the trail or step it is under,
/// and every rule and step under it.
#[derive(Clone)]
pub(crate) struct Item {
    pub id: String,
    pub open: bool,
    pub parent: Option<String>,
    pub under: Vec<String>,
}

impl Item {
    /// The first 8 characters of its id; none for a rules or closed line.
    pub fn copy(&self) -> Option<String> {
        let group = self.id.starts_with(RULES) || self.id.starts_with(CLOSED);
        (!group).then(|| short(&self.id))
    }
}

/// What `tree` with no id prints while it has no trail to print.
pub(crate) const EMPTY: &str = "No active trails. Run `celis tree --all` to see every trail.";

/// The steps and decisions a tree is printed from, the day groups it keeps, and how: grouped by
/// day and project; `width` wide; as `view` shows it, off a watch as the agent view. `next` holds the
/// earliest time a time it printed changes, `spun` whether it printed the spinner, `order` the lines a
/// watch selects, `printed` the count of its lines, and `picked` the line of the selected trail.
pub(crate) struct Tree<'a> {
    pub steps: &'a Steps<'a>,
    pub decisions: &'a Decisions<'a>,
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

/// A live rule: its record and its decision.
type Rule<'a> = (&'a celis_nostr::Note, celis_trails::Decision);

/// What the tree says of a trail: the steps it prints under it, the done ones, the time they worked,
/// and its rules.
struct Summary<'a> {
    all: usize,
    done: usize,
    worked: Option<u64>,
    rules: Vec<Rule<'a>>,
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

    /// Prints the day group of `trails` from the day `name` of `at` as `day` does, or as it last did
    /// when its layout kept it and nothing it shows changed.
    fn group(
        &self,
        (at, name): (u64, String),
        trails: Vec<Step>,
        out: &mut Vec<Styled>,
    ) -> Result<(), Failure> {
        let Some(layout) = self.layout else {
            return self.day(at, trails, out);
        };
        let key = Key {
            trails: trails.iter().map(|s| s.id().to_string()).collect(),
            width: self.width,
            full: self.view.full,
            all: self.view.all,
        };
        if let Some(group) = layout.get(&name, &key, self.view, self.steps.now()) {
            if cfg!(debug_assertions) {
                let blank = self.blank();
                let mut text = Vec::new();
                blank.day(at, trails, &mut text)?;
                assert_eq!(
                    blank.frame(text).text,
                    group.part.text,
                    "a kept group is old"
                );
            }
            return self.add(&group.part, out);
        }
        let blank = self.blank();
        let mut text = Vec::new();
        blank.day(at, trails, &mut text)?;
        let part = blank.frame(text);
        self.add(&part, out)?;
        let ids: HashSet<String> = part
            .order
            .iter()
            .flat_map(|i| std::iter::once(&i.id).chain(&i.under))
            .cloned()
            .collect();
        let group = Group {
            key,
            flipped: layout::flipped(&ids, self.view),
            selected: layout::selected(&ids, self.view),
            ids,
            part,
        };
        layout.keep(name, group);
        Ok(())
    }

    /// A tree like it that has printed nothing.
    fn blank(&self) -> Tree<'_> {
        Tree {
            spun: Cell::default(),
            order: RefCell::default(),
            printed: Cell::default(),
            picked: Cell::default(),
            next: Cell::default(),
            ..*self
        }
    }

    /// What it printed as `text`.
    fn frame(self, text: Vec<Styled>) -> Frame {
        Frame {
            text,
            next: self.next.get(),
            spun: self.spun.get(),
            order: self.order.take(),
            picked: self.picked.get(),
        }
    }

    /// Prints `part` after what it printed.
    fn add(&self, part: &Frame, out: &mut Vec<Styled>) -> Result<(), Failure> {
        out.extend(part.text.iter().cloned());
        self.order.borrow_mut().extend(part.order.iter().cloned());
        if let (None, Some(picked)) = (self.picked.get(), part.picked) {
            self.picked.set(Some(self.printed.get() + picked));
        }
        self.printed.set(self.printed.get() + part.text.len());
        if let Some(next) = part.next {
            self.until(next);
        }
        self.spun.set(self.spun.get() || part.spun);
        Ok(())
    }

    /// Prints the day group of `trails` from the day of `at`: a line with the day and its counts, then
    /// its active trails, then the rest by project.
    fn day(&self, at: u64, trails: Vec<Step>, out: &mut Vec<Styled>) -> Result<(), Failure> {
        let active = trails.iter().filter(|s| s.state().active()).count();
        let counts = format!(" · {active} active · {} done", trails.len() - active);
        let head = Styled::from(vec![paint(BOLD, &weekday(at)), paint(DIM, &counts)]);
        let mut lines = vec![Row::Mark(false, Box::new(Row::Text(head)))];
        let (active, rest): (Vec<Step>, Vec<Step>) =
            trails.into_iter().partition(|s| s.state().active());
        let mut last = None;
        if !active.is_empty() {
            lines.push(self.gap());
        }
        for note in active {
            let block = self.view.full || !self.view.all;
            self.trail(note, block, &mut last, &mut lines);
        }
        let mut projects: Vec<(&str, Vec<Step>)> = vec![];
        for note in rest {
            let project = note.record().anchor.project.as_str();
            match projects.iter_mut().find(|(p, _)| *p == project) {
                Some((_, trails)) => trails.push(note),
                None => projects.push((project, vec![note])),
            }
        }
        for (_, trails) in projects {
            let name = &trails[0].record().anchor.name;
            lines.push(self.gap());
            lines.push(Row::Mark(
                false,
                Box::new(Row::Text(Styled::from(paint(PROJECT, name)))),
            ));
            let mut last = None;
            for note in trails {
                let block = self.view.full || (!self.view.all && !self.view.watch);
                self.trail(note, block, &mut last, &mut lines);
            }
        }
        self.show(lines, out)
    }

    /// A blank line, which no element holds.
    fn gap(&self) -> Row {
        Row::Mark(false, Box::new(Row::Text(Styled::default())))
    }

    /// Adds the trail `note` as a block when `block`, else as one line, the other way when the view
    /// flipped it, in the agent view off a watch; a blank line comes between it and a `last` trail when
    /// either is a block in a watch.
    fn trail(&self, note: Step, block: bool, last: &mut Option<bool>, lines: &mut Vec<Row>) {
        let block = block != self.view.flipped.contains(note.id());
        if self.view.watch && last.is_some_and(|last| last || block) {
            lines.push(self.gap());
        }
        match (self.view.watch, block) {
            (true, true) => self.block(note, lines),
            (true, false) => self.line(note, lines),
            (false, true) => self.agent_block(note, lines),
            (false, false) => self.agent_line(note, lines),
        }
        *last = Some(block);
    }

    /// Adds `item` to the order of the lines a watch selects, and starts at `first` of `lines` the lines
    /// of its element, selected when the view selected it.
    fn pick(&self, item: Item, lines: &mut [Row], first: usize) {
        let selected = self.view.selected.as_ref() == Some(&item.id);
        let line = std::mem::replace(&mut lines[first], Row::Text(Styled::default()));
        lines[first] = Row::Mark(selected, Box::new(line));
        self.order.borrow_mut().push(item);
    }

    /// The trail `note` as a line a watch selects, `open` when it prints as a block: with every rule of
    /// `rules` and every step it prints under it.
    fn item(&self, note: Step, open: bool, rules: &[Rule]) -> Item {
        let folded = (!rules.is_empty()).then(|| format!("{RULES}{}", note.id()));
        let rules = rules.iter().map(|(rule, _)| rule.id.clone());
        let steps = Level::of(Trail::new(note).parts()).ids(note.id());
        Item {
            id: note.id().to_string(),
            open,
            parent: None,
            under: folded.into_iter().chain(rules).chain(steps).collect(),
        }
    }

    /// The lead of a line of a level after `rail`, `├─ `, or `└─ ` when it is the `last`, its `─` a `▸`
    /// or `▾` as `open` says when it opens, and the rail of the level under it.
    fn branch(&self, rail: &Styled, last: bool, open: Option<bool>) -> (Styled, Styled) {
        let (corner, under) = match last {
            true => ('└', "   "),
            false => ('├', "│  "),
        };
        let chevron = match open {
            None => '─',
            Some(false) => '▸',
            Some(true) => '▾',
        };
        (
            after(rail, paint(DIM, &format!("{corner}{chevron} "))),
            after(rail, paint(DIM, under)),
        )
    }

    /// Whether opening `note` shows something: lines under it, or steps under a closed one.
    fn opens(&self, note: Step) -> bool {
        let lines = note.record().detail().next().is_some()
            || note.done().is_some()
            || note.dropped().is_some()
            || !self.reuse.sources(note).is_empty()
            || self.reuse.followers(note) > 0;
        lines || (note.state().closed() && !note.children().is_empty())
    }

    /// Whether the step or rule `id` shows what `--full` shows: with `--full`, unless the watch closed
    /// it; else when the watch opened it.
    fn open(&self, id: &str) -> bool {
        self.view.full != self.view.flipped.contains(id)
    }

    /// Adds the object of the step `note`, named `key`, then each step under it as its own.
    fn json(&self, note: Step, key: &str, lines: &mut Vec<Row>) {
        let step = note.record();
        let mut head = json!({"id": note.id(), "created_at": note.note().created_at,
            "project": step.anchor.project, "content": step.content,
            "links": crate::text::json::links(step), "places": step.places, "session": step.session});
        self.closing_value(&mut head, note, self.work(note));
        let reworked: Vec<&str> = note.reworked().iter().map(|s| s.id()).collect();
        head["revised"] = json!(reworked);
        lines.push(Row::Text(json!({ key: head }).to_string().into()));
        self.objects(note.children(), lines);
    }

    /// Adds the object of each step of `notes`, each followed by its own.
    fn objects(&self, notes: Vec<Step>, lines: &mut Vec<Row>) {
        for note in notes {
            let mut value = crate::text::json::step(note.note(), note.record());
            self.closing_value(&mut value, note, note);
            lines.push(Row::Text(json!({ "step": value }).to_string().into()));
            self.objects(note.children(), lines);
        }
    }

    /// Adds to `value` the state of `note`, and the proof, change, grade, reuse and reason of `work`.
    fn closing_value(&self, value: &mut serde_json::Value, note: Step, work: Step) {
        let done = work.done();
        value["state"] = json!(note.state().text());
        value["proof"] = json!(done.map(|d| d.content.clone()));
        value["changed"] = json!(work.changed());
        value["grade"] = json!(Grade::of(work));
        let by: Vec<&str> = work.revised().iter().map(|s| s.id()).collect();
        value["revised_by"] = json!(by);
        value["followed"] = json!(self.reuse.followers(work));
        value["reason"] = json!(work.dropped().map(|d| d.content.clone()));
    }

    /// The step that holds the work of `note`: a trail's first ask with its text, else `note`.
    fn work<'b>(&self, note: Step<'b>) -> Step<'b> {
        match note.closed() {
            Some(_) => note,
            None => Trail::new(note).work(),
        }
    }

    /// What the tree says of the trail `note`, whose work `work` holds: the steps it prints under it, the
    /// done ones, the time they worked, and its rules. An active one lowers `next` to its end.
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
            rules: self.rules(note),
        }
    }

    /// The live rules of the trail of `note`: its own, or its trail's when it is an ask.
    fn rules(&self, note: Step) -> Vec<Rule<'_>> {
        let owner = match note.is_ask() {
            true => note.trail(),
            false => note,
        };
        self.decisions.rules(owner)
    }

    /// Adds `note` as a block: `▾`, its mark and title, then its lines: its worked time, its last change
    /// and its project while active, its project with no grouping, and the lines `--full` prints but its
    /// grade; then its tree: its active steps, its rules, then its closed steps.
    fn block(&self, note: Step, lines: &mut Vec<Row>) {
        let step = note.record();
        let state = note.state();
        let work = self.work(note);
        let summary = self.summary(note, work);
        let hair = Styled::from(vec![Span::raw("  "), paint(DIM, "│ ")]);
        let first = lines.len();
        lines.push(Row::Titled {
            lead: Styled::from(vec![paint(DIM, "▾"), Span::raw(" ")]),
            marks: self.marked(note),
            title: step.title().to_string(),
            tone: self.tone(BOLD, note.newest()),
            rest: hair.clone(),
        });
        self.pick(self.item(note, true, &summary.rules), lines, first);
        if let Some(worked) = summary.worked {
            self.labelled(lines, &hair, "took", &took(worked));
        }
        if self.view.spin && state.active() {
            let seconds = self.steps.now().saturating_sub(note.newest());
            self.until(self.steps.now() + stride(seconds));
            self.labelled(lines, &hair, "active", &format!("{} ago", took(seconds)));
        }
        if !self.grouped || state.active() {
            lines.push(Row::Note {
                rail: hair.clone(),
                label: "",
                text: step.anchor.name.clone(),
                tone: PROJECT,
            });
        }
        self.closing(work, &hair, false, lines);
        self.followed(work, &hair, lines);
        let level = Level::of(Trail::new(note).parts());
        self.level(level, summary.rules, &Styled::from("  "), note.id(), lines);
    }

    /// Adds `level` under the line `above` after `rail`: its active steps, the rules line of `rules`,
    /// then its closed line.
    fn level(
        &self,
        level: Level,
        rules: Vec<Rule>,
        rail: &Styled,
        above: &str,
        lines: &mut Vec<Row>,
    ) {
        let more = !rules.is_empty() || !level.ended.is_empty();
        self.steps(level.active, rail, above, more, lines);
        if !rules.is_empty() {
            self.rules_line(rules, rail, level.ended.is_empty(), above, lines);
        }
        if !level.ended.is_empty() {
            self.closed_line(level.ended, rail, above, lines);
        }
    }

    /// Adds the rules line `§ <n> rules` of the trail `trail` after `rail`, the `last` of its level;
    /// open, a line under it for each of `rules`: `§ <text>`, wrapped, its id at the end.
    fn rules_line(
        &self,
        rules: Vec<Rule>,
        rail: &Styled,
        last: bool,
        trail: &str,
        lines: &mut Vec<Row>,
    ) {
        let id = format!("{RULES}{trail}");
        let open = self.open(&id);
        let (lead, rail) = self.branch(rail, last, Some(open));
        let mark = paint(MAGENTA, "§");
        let first = lines.len();
        lines.push(Row::Titled {
            lead,
            marks: Styled::from(mark.clone()),
            title: plural(rules.len(), "rule"),
            tone: Style::new(),
            rest: after(&rail, Span::raw("  ")),
        });
        let item = Item {
            id: id.clone(),
            open,
            parent: Some(trail.to_string()),
            under: rules.iter().map(|(rule, _)| rule.id.clone()).collect(),
        };
        self.pick(item, lines, first);
        if !open {
            return;
        }
        let all = rules.len();
        for (n, (rule, decision)) in rules.into_iter().enumerate() {
            let (lead, below) = self.branch(&rail, n + 1 == all, None);
            let first = lines.len();
            lines.push(Row::Titled {
                lead,
                marks: Styled::from(mark.clone()),
                title: decision.content.clone(),
                tone: Style::new(),
                rest: after(&below, Span::raw("  ")),
            });
            let item = Item {
                id: rule.id.clone(),
                open: false,
                parent: Some(id.clone()),
                under: vec![],
            };
            self.pick(item, lines, first);
        }
    }

    /// Adds the closed line `<marks> <n> closed steps` of the ended `steps` of the line `above`, the last
    /// of its level after `rail`: `✓` when one is done, `✗` when one is dropped, `◌` when one is expired;
    /// open, the steps under it.
    fn closed_line(&self, steps: Vec<Step>, rail: &Styled, above: &str, lines: &mut Vec<Row>) {
        let id = format!("{CLOSED}{above}");
        let open = self.open(&id);
        let (lead, rail) = self.branch(rail, true, Some(open));
        let tally = Tally::of(&steps);
        let marks: Styled = [State::Done, State::Dropped, State::Expired]
            .into_iter()
            .filter(|state| tally.count(*state) > 0)
            .map(|state| paint(look(state).text, look(state).mark))
            .collect();
        let first = lines.len();
        lines.push(Row::Titled {
            lead,
            marks,
            title: plural(steps.len(), "closed step"),
            tone: Style::new(),
            rest: after(&rail, Span::raw("  ")),
        });
        let item = Item {
            id: id.clone(),
            open,
            parent: Some(above.to_string()),
            under: ids(&steps),
        };
        self.pick(item, lines, first);
        if open {
            self.steps(steps, &rail, &id, false, lines);
        }
    }

    /// Adds `note` as one row: `▸`, its mark and wrapped title, then its details: its steps and worked
    /// time.
    fn line(&self, note: Step, lines: &mut Vec<Row>) {
        let step = note.record();
        let state = note.state();
        let work = self.work(note);
        let summary = self.summary(note, work);
        let steps = match (summary.all, summary.done) {
            (0, _) => None,
            (all, done) if all == done => Some(plural(all, "step")),
            (all, done) => Some(format!("{done} of {all} steps")),
        };
        let first = lines.len();
        let rest = Styled::from("    ");
        lines.push(Row::Titled {
            lead: Styled::from(vec![paint(DIM, "▸"), Span::raw(" ")]),
            marks: self.marked(note),
            title: step.title().to_string(),
            tone: self.tone(look(state).title, note.newest()),
            rest: rest.clone(),
        });
        self.details(&rest, [steps, summary.worked.map(took)], lines);
        self.pick(self.item(note, false, &summary.rules), lines, first);
    }

    /// Adds `details` after `rail` as one dim line, `·` between two; none when all are empty.
    fn details(&self, rail: &Styled, details: [Option<String>; 2], lines: &mut Vec<Row>) {
        let shown: Vec<String> = details.into_iter().flatten().collect();
        if !shown.is_empty() {
            lines.push(Row::Note {
                rail: rail.clone(),
                label: "",
                text: shown.join(" · "),
                tone: DIM,
            });
        }
    }

    /// Adds the steps `notes` under the line `above` after `rail`, the last of their level unless `more`,
    /// each followed by its own one level down, each with its title wrapped and its details under it; an
    /// open one with the lines `--full` prints under it, a closed done one with no time and its steps
    /// folded.
    fn steps(
        &self,
        notes: Vec<Step>,
        rail: &Styled,
        above: &str,
        more: bool,
        lines: &mut Vec<Row>,
    ) {
        let all = notes.len();
        for (n, note) in notes.into_iter().enumerate() {
            let step = note.record();
            let state = note.state();
            if state.active() {
                self.until(note.trail().newest() + LIFE);
            }
            let open = self.open(note.id());
            let last = n + 1 == all && !more;
            let (lead, rail) = self.branch(rail, last, self.opens(note).then_some(open));
            let size = note.tree().len() - 1;
            let level = Level::of(note.children());
            let folded = state == State::Done && size > 0 && !open;
            let below = match note.children().is_empty() || folded {
                true => after(&rail, Span::raw("  ")),
                false => after(&rail, paint(DIM, "│ ")),
            };
            let first = lines.len();
            lines.push(Row::Titled {
                lead,
                marks: self.marked(note),
                title: step.title().to_string(),
                tone: self.tone(look(state).title, note.newest()),
                rest: below.clone(),
            });
            let time = match state.closed() {
                true => None,
                false => self.worked(&note.tree()).map(took),
            };
            self.details(&below, [folded.then(|| plural(size, "step")), time], lines);
            let item = Item {
                id: note.id().to_string(),
                open,
                parent: Some(above.to_string()),
                under: level.ids(note.id()),
            };
            self.pick(item, lines, first);
            if open {
                self.closing(note, &below, true, lines);
                self.followed(note, &below, lines);
            }
            if !folded {
                self.level(level, vec![], &rail, note.id(), lines);
            }
        }
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

    /// Adds under `note` the lines of its text, its proof, change and with `grade` its grade when done, or
    /// its reason when dropped, after `rail`.
    fn closing(&self, note: Step, rail: &Styled, grade: bool, out: &mut Vec<Row>) {
        for line in note.record().detail() {
            self.labelled(out, rail, "", line);
        }
        if let (Some(done), Some(line)) = (note.done(), crate::text::line::grade(note)) {
            self.labelled(out, rail, "done", &done.content);
            if let Some(changed) = done.tag("changed") {
                self.labelled(out, rail, "changed", changed);
            }
            if grade {
                self.labelled(out, rail, "took", line.trim_start_matches("took "));
            }
        }
        if let Some(reason) = note.dropped() {
            self.labelled(out, rail, "dropped", &reason.content);
        }
    }

    /// Adds under `note` a `follows` line for each step it follows, then a `followed` line when steps follow
    /// it, after `rail`.
    fn followed(&self, note: Step, rail: &Styled, out: &mut Vec<Row>) {
        for source in self.reuse.sources(note) {
            let line = format!("{} {}", short(source.id()), source.title());
            self.labelled(out, rail, "follows", &line);
        }
        let count = self.reuse.followers(note);
        if count > 0 {
            self.labelled(out, rail, "followed", &count.to_string());
        }
    }

    /// Adds `text` dim after `rail` and `label` padded to 8 characters, or after `rail` alone with no
    /// label.
    fn labelled(&self, lines: &mut Vec<Row>, rail: &Styled, label: &'static str, text: &str) {
        lines.push(Row::Note {
            rail: rail.clone(),
            label,
            text: text.to_string(),
            tone: DIM,
        });
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
/// What starts the id of a trail's rules line in a watch's order.
const RULES: &str = "§";
/// The id of the `Show more` line in a watch's order.
pub(crate) const MORE: &str = "+more";
/// What starts the id of a trail's closed line in a watch's order.
const CLOSED: &str = "✓";
/// The colour of a project's name.
const PROJECT: Style = BOLD;
mod agent;
mod layout;
mod level;
mod row;
#[cfg(test)]
mod tests;
mod watch;
