//! `celis tree`: a trail with its state, its rules and the steps under it.

use super::clock::{day, midnight, weekday};
use super::clock::{saved, stride, took};
use super::{Args, Command};
use celis_trails::id::short;
use celis_trails::step::steps::LIFE;
use celis_trails::{Decisions, Failure, Flags, Reuse, State, Step, Steps, Tally, Trail, Trails};
use level::{ids, Level};
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
        if args.demo && id.is_some() {
            return Err(Failure::usage("tree takes an id or --demo, not both"));
        }
        if args.demo {
            return self.demo(args, out);
        }
        if std::io::stdout().is_terminal() && !args.json {
            return self.watch(args, out);
        }
        let tree = match (self.held()?, id) {
            (true, _) => {
                let view = View::of(args);
                self.frame(&self.trails()?, args, None, &view)?.text
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
        let steps = trails.steps();
        let decisions = trails.decisions();
        let reuse = Reuse::new(&steps);
        let chosen: Vec<Step> = match (id, view.all) {
            (Some(id), _) => vec![steps.find(id)?],
            (None, true) => steps.roots().into_iter().rev().collect(),
            (None, false) => today(&steps, trails.now),
        };
        let tree = Tree {
            steps: &steps,
            decisions: &decisions,
            reuse: &reuse,
            colour: view.watch && self.colour(),
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
        Ok(Frame {
            text: String::from_utf8_lossy(&printed).into_owned(),
            next: tree.next.get(),
            spun: tree.spun.get(),
            order: tree.order.take(),
            picked: tree.picked.get(),
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

/// How a tree shows: in a watch, with a cursor column; full; all with no id; and in a watch, the
/// spinner's frame, the lines it opened or closed, and the line it selected.
#[derive(Default)]
pub(super) struct View {
    pub watch: bool,
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
/// the lines a watch selects in the order it prints them, and the line of the selected one.
pub(super) struct Frame {
    pub text: String,
    pub next: Option<u64>,
    pub spun: bool,
    pub order: Vec<Item>,
    pub picked: Option<usize>,
}

/// A trail, rule or step a watch selects: its id, whether it is open, the trail or step it is under,
/// and every rule and step under it.
pub(super) struct Item {
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
pub(super) const EMPTY: &str = "No active trails. Run `celis tree --all` to see every trail.";

/// The steps and decisions a tree is printed from, and how: in colour; grouped by day and project; each
/// line cut to `width`; as `view` shows it, off a watch as the agent view. `next` holds the earliest time a time it printed
/// changes, `spun` whether it printed the spinner, `order` the lines a watch selects, `printed` the count of
/// its lines, and `picked` the line of the selected trail.
pub(super) struct Tree<'a> {
    pub steps: &'a Steps<'a>,
    pub decisions: &'a Decisions<'a>,
    pub reuse: &'a Reuse<'a>,
    pub colour: bool,
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

/// A line of the tree: text as it is, or a row whose details line up with the other rows' of its day,
/// its title in the colour `tone`, wrapped with its next lines after `rest` when it has one; or the
/// start of a selected element's lines, or of lines no element holds.
enum Line {
    Text(String),
    Row {
        lead: String,
        title: String,
        tone: &'static str,
        cells: [Option<String>; 5],
        rest: Option<String>,
    },
    Mark(bool, Box<Line>),
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
        out: &mut dyn Write,
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
        let mut days: Vec<(u64, Vec<Step>)> = vec![];
        for &note in notes {
            let at = match self.view.all {
                true => note.note().created_at,
                false => self.steps.now,
            };
            match days.last_mut() {
                Some((first, trails)) if day(*first) == day(at) => trails.push(note),
                _ => days.push((at, vec![note])),
            }
        }
        for (n, (at, trails)) in days.into_iter().enumerate() {
            if n > 0 {
                self.put(out, "")?;
            }
            let active = trails.iter().filter(|s| s.state().active()).count();
            let counts = format!(" · {active} active · {} done", trails.len() - active);
            let head = format!(
                "{}{}",
                self.paint(BOLD, &weekday(at)),
                self.paint(DIM, &counts)
            );
            let mut lines = vec![Line::Mark(false, Box::new(Line::Text(head)))];
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
                lines.push(Line::Mark(
                    false,
                    Box::new(Line::Text(self.paint(PROJECT, name))),
                ));
                let mut last = None;
                for note in trails {
                    let block = self.view.full || (!self.view.all && !self.view.watch);
                    self.trail(note, block, &mut last, &mut lines);
                }
            }
            self.show(lines, out)?;
        }
        if self.view.watch && !notes.is_empty() {
            self.put(out, "")?;
            let (chevron, label) = match self.view.all {
                true => ("▾", "Show less"),
                false => ("▸", "Show more"),
            };
            let mut lines = vec![Line::Row {
                lead: format!("{} ", self.paint(DIM, chevron)),
                title: label.to_string(),
                tone: "",
                cells: Default::default(),
                rest: None,
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

    /// A blank line, which no element holds.
    fn gap(&self) -> Line {
        Line::Mark(false, Box::new(Line::Text(String::new())))
    }

    /// Adds the trail `note` as a block when `block`, else as one line, the other way when the view
    /// flipped it, in the agent view off a watch; a blank line comes between it and a `last` trail when
    /// either is a block in a watch.
    fn trail(&self, note: Step, block: bool, last: &mut Option<bool>, lines: &mut Vec<Line>) {
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
    fn pick(&self, item: Item, lines: &mut [Line], first: usize) {
        let selected = self.view.selected.as_ref() == Some(&item.id);
        let line = std::mem::replace(&mut lines[first], Line::Text(String::new()));
        lines[first] = Line::Mark(selected, Box::new(line));
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
    fn branch(&self, rail: &str, last: bool, open: Option<bool>) -> (String, String) {
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
            format!("{rail}{}", self.paint(DIM, &format!("{corner}{chevron} "))),
            format!("{rail}{}", self.paint(DIM, under)),
        )
    }

    /// Whether opening `note` shows something: lines under it, or steps under a closed one.
    fn opens(&self, note: Step) -> bool {
        let lines = note.record().detail().next().is_some()
            || note.done().is_some()
            || note.dropped().is_some()
            || !self.reuse.sources(note).is_empty()
            || !self.reuse.down(note).is_empty();
        lines || (note.state().closed() && !note.children().is_empty())
    }

    /// Whether the step or rule `id` shows what `--full` shows: with `--full`, unless the watch closed
    /// it; else when the watch opened it.
    fn open(&self, id: &str) -> bool {
        self.view.full != self.view.flipped.contains(id)
    }

    /// Adds the object of the step `note`, named `key`, then each step under it as its own.
    fn json(&self, note: Step, key: &str, lines: &mut Vec<Line>) {
        let step = note.record();
        let mut head = json!({"id": note.id(), "created_at": note.note().created_at,
            "project": step.anchor.project, "content": step.content,
            "links": super::json::links(step), "places": step.places, "session": step.session});
        self.closing_value(&mut head, note, self.work(note));
        let reworked: Vec<&str> = note.reworked().iter().map(|s| s.id()).collect();
        head["revised"] = json!(reworked);
        lines.push(Line::Text(json!({ key: head }).to_string()));
        self.objects(note.children(), lines);
    }

    /// Adds the object of each step of `notes`, each followed by its own.
    fn objects(&self, notes: Vec<Step>, lines: &mut Vec<Line>) {
        for note in notes {
            let mut value = super::json::step(note.note(), note.record());
            self.closing_value(&mut value, note, note);
            lines.push(Line::Text(json!({ "step": value }).to_string()));
            self.objects(note.children(), lines);
        }
    }

    /// Adds to `value` the state of `note`, and the proof, change, grade, reuse and reason of `work`.
    fn closing_value(&self, value: &mut serde_json::Value, note: Step, work: Step) {
        let done = work.done();
        value["state"] = json!(note.state().text());
        value["proof"] = json!(done.map(|d| d.content.clone()));
        value["changed"] = json!(work.changed());
        value["grade"] = json!(work.grade());
        let by: Vec<&str> = work.revised().iter().map(|s| s.id()).collect();
        value["revised_by"] = json!(by);
        value["reused"] = json!(self.reuse.down(work).len());
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
        self.decisions
            .rules(&note.record().anchor.project, owner.id())
    }

    /// Adds `note` as a block: `▾`, its mark and title, then its lines: its worked time, its last change
    /// while active, its project with no grouping, and the lines `--full` prints but its grade; then its
    /// tree: its active steps, its rules, then its closed steps.
    fn block(&self, note: Step, lines: &mut Vec<Line>) {
        let step = note.record();
        let state = note.state();
        let work = self.work(note);
        let summary = self.summary(note, work);
        let hair = format!("  {}", self.paint(DIM, "│ "));
        let project =
            (self.grouped && state.active()).then(|| self.paint(PROJECT, &step.anchor.name));
        let first = lines.len();
        lines.push(Line::Row {
            lead: format!("{} {} ", self.paint(DIM, "▾"), self.marked(note)),
            title: step.title().to_string(),
            tone: self.tone(BOLD, note.newest()),
            cells: [project, None, None, None, None],
            rest: Some(hair.clone()),
        });
        self.pick(self.item(note, true, &summary.rules), lines, first);
        if let Some(worked) = summary.worked {
            self.labelled(lines, &hair, "took", &took(worked));
        }
        if self.view.spin.is_some() && state.active() {
            let seconds = self.steps.now.saturating_sub(note.newest());
            self.until(self.steps.now + stride(seconds));
            self.labelled(lines, &hair, "active", &format!("{} ago", took(seconds)));
        }
        if !self.grouped {
            let project = self.paint(PROJECT, &step.anchor.name);
            lines.push(Line::Text(format!("{hair}{project}")));
        }
        self.closing(work, &hair, false, lines);
        self.reused(work, &hair, lines);
        let level = Level::of(Trail::new(note).parts());
        self.level(level, summary.rules, "  ", note.id(), lines);
    }

    /// Adds `level` under the line `above` after `rail`: its active steps, the rules line of `rules`,
    /// then its closed line.
    fn level(
        &self,
        level: Level,
        rules: Vec<Rule>,
        rail: &str,
        above: &str,
        lines: &mut Vec<Line>,
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
        rail: &str,
        last: bool,
        trail: &str,
        lines: &mut Vec<Line>,
    ) {
        let id = format!("{RULES}{trail}");
        let open = self.open(&id);
        let (lead, rail) = self.branch(rail, last, Some(open));
        let mark = self.paint(MAGENTA, "§");
        let first = lines.len();
        lines.push(Line::Row {
            lead: format!("{lead}{mark} "),
            title: count(rules.len(), "rule"),
            tone: "",
            cells: Default::default(),
            rest: None,
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
            lines.push(Line::Row {
                lead: format!("{lead}{mark} "),
                title: decision.content.clone(),
                tone: "",
                cells: [
                    None,
                    None,
                    None,
                    None,
                    Some(self.paint(DIM, &short(&rule.id))),
                ],
                rest: Some(format!("{below}  ")),
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
    /// of its level after `rail`: `✓` when one is done, `✗` when one is dropped, `!` when one is expired;
    /// open, the steps under it.
    fn closed_line(&self, steps: Vec<Step>, rail: &str, above: &str, lines: &mut Vec<Line>) {
        let id = format!("{CLOSED}{above}");
        let open = self.open(&id);
        let (lead, rail) = self.branch(rail, true, Some(open));
        let tally = Tally::of(&steps);
        let marks: String = [State::Done, State::Dropped, State::Expired]
            .into_iter()
            .filter(|state| tally.count(*state) > 0)
            .map(|state| self.paint(look(state).text, look(state).mark))
            .collect();
        let first = lines.len();
        lines.push(Line::Row {
            lead: format!("{lead}{marks} "),
            title: count(steps.len(), "closed step"),
            tone: "",
            cells: Default::default(),
            rest: None,
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

    /// Adds `note` as one line: `▸`, its mark and title, then its steps, rules and worked time.
    fn line(&self, note: Step, lines: &mut Vec<Line>) {
        let step = note.record();
        let state = note.state();
        let work = self.work(note);
        let summary = self.summary(note, work);
        let steps = match (summary.all, summary.done) {
            (0, _) => None,
            (all, done) if all == done => Some(column(all, "step")),
            (all, done) => Some(format!("{done} of {all}")),
        };
        let rules = summary.rules.len();
        let first = lines.len();
        lines.push(Line::Row {
            lead: format!("{} {} ", self.paint(DIM, "▸"), self.marked(note)),
            title: step.title().to_string(),
            tone: self.tone(look(state).title, note.newest()),
            cells: [
                None,
                steps,
                (rules > 0).then(|| column(rules, "rule")),
                summary.worked.map(took),
                Some(self.id(note)),
            ],
            rest: None,
        });
        self.pick(self.item(note, false, &summary.rules), lines, first);
    }

    /// Adds the steps `notes` under the line `above` after `rail`, the last of their level unless `more`,
    /// each followed by its own one level down; an open one with its title wrapped and the lines `--full`
    /// prints under it, a closed done one with no time and its steps folded.
    fn steps(&self, notes: Vec<Step>, rail: &str, above: &str, more: bool, lines: &mut Vec<Line>) {
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
                true => format!("{rail}  "),
                false => format!("{rail}{}", self.paint(DIM, "│ ")),
            };
            let first = lines.len();
            lines.push(Line::Row {
                lead: format!("{lead}{} ", self.marked(note)),
                title: step.title().to_string(),
                tone: self.tone(look(state).title, note.newest()),
                cells: [
                    None,
                    folded.then(|| column(size, "step")),
                    None,
                    match state.closed() {
                        true => None,
                        false => self.worked(&note.tree()).map(took),
                    },
                    Some(self.id(note)),
                ],
                rest: open.then(|| below.clone()),
            });
            let item = Item {
                id: note.id().to_string(),
                open,
                parent: Some(above.to_string()),
                under: level.ids(note.id()),
            };
            self.pick(item, lines, first);
            if open {
                self.closing(note, &below, true, lines);
                self.reused(note, &below, lines);
            }
            if !folded {
                self.level(level, vec![], &rail, note.id(), lines);
            }
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
    /// title; in the short tree, a row's title is cut so it fits, and a narrow width leaves out the first
    /// column, then the time, while a title would be cut below 20 characters. In a watch, the ids are left
    /// out, and each line starts with the cursor column, a bar on each line of the selected element.
    fn show(&self, lines: Vec<Line>, out: &mut dyn Write) -> Result<(), Failure> {
        let mut widths = [0; 5];
        for (_, _, cells, _) in lines.iter().filter_map(row) {
            for (width, cell) in widths.iter_mut().zip(cells) {
                *width = (*width).max(cell.as_deref().map_or(0, visible));
            }
        }
        if self.view.watch {
            widths[4] = 0;
        }
        if let (false, Some(width)) = (self.view.full, self.width) {
            let first = widths.iter().position(|w| *w > 0).unwrap_or(0);
            for column in [first, 3] {
                let short = lines.iter().filter_map(row).any(|(lead, title, _, _)| {
                    width.saturating_sub(visible(lead) + room(&widths)) < visible(title).min(20)
                });
                if short {
                    widths[column] = 0;
                }
            }
        }
        let room = room(&widths);
        let edge = match (self.view.full, self.width) {
            (false, Some(width)) => width,
            (_, width) => lines
                .iter()
                .filter_map(row)
                .map(|(lead, title, _, wraps)| {
                    let title = match (wraps, width) {
                        (true, Some(w)) => {
                            visible(title).min(w.saturating_sub(visible(lead) + room))
                        }
                        _ => visible(title),
                    };
                    visible(lead) + title + room
                })
                .chain(width)
                .max()
                .unwrap_or(0),
        };
        let mut picked = false;
        for line in lines {
            let line = match line {
                Line::Mark(selected, line) => {
                    picked = selected;
                    if picked && self.picked.get().is_none() {
                        self.picked.set(Some(self.printed.get()));
                    }
                    *line
                }
                line => line,
            };
            for text in self.render(line, &widths, edge, picked) {
                let text = match (self.view.watch, picked) {
                    (false, _) => text,
                    (true, false) => format!("  {text}"),
                    (true, true) => format!("▌ {text}"),
                };
                self.put(out, &text)?;
            }
        }
        Ok(())
    }

    /// The lines of `line`: a row's title, bold when `picked` unless dim, cut in the short tree or wrapped
    /// when it has a rest, then its details in the columns of `widths` up to its last one, each dim
    /// unless it has its own colour, ending at `edge`.
    fn render(&self, line: Line, widths: &[usize; 5], edge: usize, picked: bool) -> Vec<String> {
        let (lead, title, tone, cells, rest) = match line {
            Line::Row {
                lead,
                title,
                tone,
                cells,
                rest,
            } => (lead, title, tone, cells, rest),
            Line::Text(text) => return vec![text],
            Line::Mark(_, line) => return self.render(*line, widths, edge, picked),
        };
        let tone = match picked && tone != DIM {
            true => BOLD,
            false => tone,
        };
        let filled = |n: &usize| cells[*n].is_some() && widths[*n] > 0;
        let (from, to) = ((0..5).find(filled), (0..5).rev().find(filled));
        let mut widths = *widths;
        widths[..from.unwrap_or(5)].fill(0);
        widths[to.map_or(0, |to| to + 1)..].fill(0);
        let room = room(&widths);
        let mut titles = match (self.width, &rest, self.view.full) {
            (Some(width), Some(_), _) => {
                wrap(&title, width.saturating_sub(visible(&lead) + room).max(20))
            }
            (Some(_), None, false) => {
                vec![cut(
                    &title,
                    edge.saturating_sub(visible(&lead) + room).max(2),
                )]
            }
            _ => vec![title],
        };
        if titles.is_empty() {
            titles.push(String::new());
        }
        let block: Vec<String> = cells
            .iter()
            .zip(widths)
            .enumerate()
            .filter(|(_, (_, width))| *width > 0)
            .map(|(n, (cell, width))| {
                let cell = cell.as_deref().unwrap_or("");
                let pad = " ".repeat(width - visible(cell));
                let cell = match cell.contains('\x1b') {
                    true => cell.to_string(),
                    false => self.paint(DIM, cell),
                };
                match n {
                    1..=3 => format!("{pad}{cell}"),
                    n if Some(n) == to => format!("{pad}{cell}"),
                    _ => format!("{cell}{pad}"),
                }
            })
            .collect();
        let block = block.join("  ");
        let first = &titles[0];
        let gap = edge.saturating_sub(visible(&lead) + visible(first) + room.saturating_sub(2));
        let first = match block.trim_end() {
            "" => format!("{lead}{}", self.paint(tone, first)),
            block => format!(
                "{lead}{}{}{block}",
                self.paint(tone, first),
                " ".repeat(gap)
            ),
        };
        let rest = rest.unwrap_or_default();
        let next = titles[1..]
            .iter()
            .map(|t| format!("{rest}{}", self.paint(tone, t)));
        std::iter::once(first).chain(next).collect()
    }

    /// Writes the line `text`, and counts it.
    fn put(&self, out: &mut dyn Write, text: &str) -> Result<(), Failure> {
        self.printed.set(self.printed.get() + 1);
        super::put(out, text)
    }

    /// Adds under `note` the lines of its text, its proof, change and with `grade` its grade when done, or
    /// its reason when dropped, after `rail`.
    fn closing(&self, note: Step, rail: &str, grade: bool, out: &mut Vec<Line>) {
        for line in note.record().detail() {
            self.labelled(out, rail, "", line);
        }
        if let (Some(done), Some(line)) = (note.done(), super::line::grade(note)) {
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

    /// Adds under `note` a `follows` line for each step it follows, then a `reused` line when steps follow
    /// it, after `rail`.
    fn reused(&self, note: Step, rail: &str, out: &mut Vec<Line>) {
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

    /// Adds `text` wrapped, dim, after `rail` and `label` padded to 8 characters, or after `rail` alone
    /// with no label.
    fn labelled(&self, lines: &mut Vec<Line>, rail: &str, label: &str, text: &str) {
        let (first, rest) = match label {
            "" => (rail.to_string(), rail.to_string()),
            _ => (
                format!("{rail}{}", self.paint(DIM, &format!("{label:<8}"))),
                format!("{rail}{}", " ".repeat(8)),
            ),
        };
        self.text(lines, (first, rest), text, true, DIM);
    }

    /// Adds `text` in the colour `code` after `leads.0`, its next lines after `leads.1`: when `whole`,
    /// wrapped at the width, each line at the same column; else cut to it.
    fn text(
        &self,
        lines: &mut Vec<Line>,
        leads: (String, String),
        text: &str,
        whole: bool,
        code: &str,
    ) {
        let (first, rest) = leads;
        let column = visible(&first);
        let measure = self.width.map(|w| w.saturating_sub(column).max(20));
        let parts = match (whole, measure) {
            (true, Some(measure)) => wrap(text, measure),
            (false, Some(measure)) => vec![cut(text, measure)],
            (_, None) => vec![text.to_string()],
        };
        for (n, part) in parts.iter().enumerate() {
            let lead = if n == 0 { &first } else { &rest };
            lines.push(Line::Text(format!("{lead}{}", self.paint(code, part))));
        }
    }

    /// The mark of `note` with its flags after it, each in its colour.
    fn marked(&self, note: Step) -> String {
        let signs = signs(note.flags()).into_iter();
        let flags = signs.map(|(sign, code)| self.paint(code, sign));
        self.mark(note.state()) + &flags.collect::<String>()
    }

    /// The mark of a step in `state` in its colour; in a watch, a working one's is the spinner.
    fn mark(&self, state: State) -> String {
        let look = look(state);
        let mark = match (state, self.view.spin) {
            (State::Working, Some(frame)) => {
                self.spun.set(true);
                return self.paint(SPINNER, SPIN[frame % SPIN.len()]);
            }
            _ => look.mark,
        };
        self.paint(look.text, mark)
    }

    /// The colour of a title usually in `usual`: bold for 2 seconds after its newest record at `newest`.
    fn tone(&self, usual: &'static str, newest: u64) -> &'static str {
        if newest + FRESH <= self.steps.now {
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
/// The colour of the spinner: bright yellow.
const SPINNER: &str = "93";
const BOLD: &str = "1";
const GREEN: &str = "32";
const YELLOW: &str = "33";
const MAGENTA: &str = "35";
/// What starts the id of a trail's rules line in a watch's order.
const RULES: &str = "§";
/// The id of the `Show more` line in a watch's order.
pub(super) const MORE: &str = "+more";
/// What starts the id of a trail's closed line in a watch's order.
const CLOSED: &str = "✓";
/// The colour of a project's name.
const PROJECT: &str = BOLD;
const UNDERLINE: &str = "4;34";
/// The lead, title and details of `line` when it is a row, also when it is selected, and whether its
/// title wraps.
fn row(line: &Line) -> Option<(&String, &String, &[Option<String>; 5], bool)> {
    match line {
        Line::Row {
            lead,
            title,
            cells,
            rest,
            ..
        } => Some((lead, title, cells, rest.is_some())),
        Line::Mark(_, line) => row(line),
        Line::Text(_) => None,
    }
}

/// The characters the detail columns of `widths` take with the two spaces before them; none when all are
/// empty.
fn room(widths: &[usize; 5]) -> usize {
    let shown = widths.iter().filter(|w| **w > 0).count();
    match shown {
        0 => 0,
        n => widths.iter().sum::<usize>() + 2 * n,
    }
}

/// The signs of `flags` with their colours: `↻` when revised, then `~` when changed.
pub(super) fn signs(flags: Flags) -> Vec<(&'static str, &'static str)> {
    let all = [(flags.revised, "↻", GREEN), (flags.changed, "~", YELLOW)];
    all.into_iter()
        .filter(|(on, ..)| *on)
        .map(|(_, sign, code)| (sign, code))
        .collect()
}

/// The characters of `text` a terminal shows; colour codes count none.
pub(super) fn visible(text: &str) -> usize {
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
        State::Working => ("◐", YELLOW, BOLD),
        State::Done => ("✓", GREEN, ""),
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

/// `n` of `what` for a column whose numbers align right, as `1 step ` or `3 steps`.
fn column(n: usize, what: &str) -> String {
    match n {
        1 => format!("1 {what} "),
        n => format!("{n} {what}s"),
    }
}

mod agent;
mod level;
#[cfg(test)]
mod tests;
