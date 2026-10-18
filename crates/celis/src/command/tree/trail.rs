//! A trail in the tree: a block with its steps, its rules line and its closed line, or one line.

use super::level::{ids, Level};
use super::row::Row;
use super::{after, Item, Rule, Tree, CLOSED, PROJECT, RULES};
use crate::progress::Tally;
use crate::terminal::canvas::{paint, Styled, BOLD, DIM, MAGENTA};
use crate::terminal::look::look;
use crate::text::clock::{stride, took};
use crate::text::line::plural;
use celis_trails::records::id::short;
use celis_trails::step::steps::LIFE;
use celis_trails::{State, Step, Trail};
use ratatui::style::Style;
use ratatui::text::Span;

impl Tree<'_> {
    /// Adds the trail `note` as a block when `block`, else as one line, the other way when the view
    /// flipped it, in the agent view off a watch; a blank line comes between it and a `last` trail when
    /// either is a block in a watch.
    pub(super) fn trail(
        &self,
        note: Step,
        block: bool,
        last: &mut Option<bool>,
        lines: &mut Vec<Row>,
    ) {
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
    pub(super) fn pick(&self, item: Item, lines: &mut [Row], first: usize) {
        let selected = self.view.selected.as_ref() == Some(&item.id);
        let line = std::mem::replace(&mut lines[first], Row::Text(Styled::default()));
        lines[first] = Row::Mark(selected, Box::new(line));
        self.order.borrow_mut().push(item);
    }

    /// The trail `note` as a line a watch selects, `open` when it prints as a block: with every rule of
    /// `rules` and every step it prints under it.
    pub(super) fn item(&self, note: Step, open: bool, rules: &[Rule]) -> Item {
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
    pub(super) fn opens(&self, note: Step) -> bool {
        let lines = note.record().detail().next().is_some()
            || note.done().is_some()
            || note.dropped().is_some()
            || !self.reuse.sources(note).is_empty()
            || self.reuse.followers(note) > 0;
        lines || (note.state().closed() && !note.children().is_empty())
    }

    /// Whether the step or rule `id` shows what `--full` shows: with `--full`, unless the watch closed
    /// it; else when the watch opened it.
    pub(super) fn open(&self, id: &str) -> bool {
        self.view.full != self.view.flipped.contains(id)
    }

    /// Adds `note` as a block: `▾`, its mark and title, then its lines: its worked time, its last change
    /// and its project while active, its project with no grouping, and the lines `--full` prints but its
    /// grade; then its tree: its active steps, its rules, then its closed steps.
    pub(super) fn block(&self, note: Step, lines: &mut Vec<Row>) {
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
    pub(super) fn level(
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
    pub(super) fn line(&self, note: Step, lines: &mut Vec<Row>) {
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
    pub(super) fn details(
        &self,
        rail: &Styled,
        details: [Option<String>; 2],
        lines: &mut Vec<Row>,
    ) {
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
    pub(super) fn steps(
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

    /// Adds under `note` the lines of its text, its proof, change and with `grade` its grade when done, or
    /// its reason when dropped, after `rail`.
    pub(super) fn closing(&self, note: Step, rail: &Styled, grade: bool, out: &mut Vec<Row>) {
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
    pub(super) fn followed(&self, note: Step, rail: &Styled, out: &mut Vec<Row>) {
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
}
