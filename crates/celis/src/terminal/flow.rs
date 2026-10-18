//! The graph of one trail in the node view: the steps right under it as cards in a chain from left to
//! right, and each one's own steps as cards below it.

use super::history::{History, Kind};
use crate::command::tree::{cut, wrap};
use celis_trails::{State, Step, Trail};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Widget};
use std::cell::Cell;
use std::collections::{HashMap, HashSet};

/// The width of a card in the chain.
const CARD: u16 = 28;

/// The width of the edge between two cards of the chain.
const GAP: u16 = 5;

/// The most rows of a title in the chain.
const LINES: usize = 3;

/// The most steps a card under the chain lists.
const ROWS: usize = 4;

/// The most rules shown, the newest.
const RULES: usize = 3;

/// The frames of a working step's mark.
const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// The trail `head` after the first `shown` events of `history`, with the step `selected` lit; a
/// working step's mark at the spinner's frame `spin`, else still.
pub(crate) struct Flow<'a> {
    pub head: Step<'a>,
    pub history: &'a History,
    pub shown: usize,
    pub selected: Option<&'a str>,
    pub spin: Option<usize>,
}

/// A step as a card: where it stands, whether a later step replaces it, its place, its title rows, its
/// own steps as rows with the count left out, and the done and all steps under it.
struct Card<'a> {
    step: Step<'a>,
    state: State,
    replaced: bool,
    area: Rect,
    title: Vec<String>,
    rows: Vec<(Step<'a>, State)>,
    more: usize,
    progress: Option<(usize, usize)>,
}

impl<'a> Flow<'a> {
    /// The width and height it takes to draw the whole graph.
    pub fn size(&self) -> (u16, u16) {
        let (_, width, height) = self.layout();
        (width.max(CARD), 2 + height + self.below())
    }

    /// The ids of the steps it draws, a column for each card of the chain: its card, then each card
    /// under it followed by its rows.
    pub fn columns(&self) -> Vec<Vec<String>> {
        let (columns, _, _) = self.layout();
        let ids = |card: &Card| {
            let rows = card.rows.iter().map(|(s, _)| s.id().to_string());
            std::iter::once(card.step.id().to_string())
                .chain(rows)
                .collect::<Vec<_>>()
        };
        let column = |cards: &Vec<Card>| cards.iter().flat_map(ids).collect();
        columns.iter().map(column).collect()
    }

    /// Draws the header, the whole columns of the graph that fit from the column `offset`, moved just
    /// enough to show the selected step, else the newest event's step, and the newest rules, over `area` of `buf`; keeps the
    /// first column in `offset`. The area of each step, edge and rule it drew, by its id.
    pub fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        offset: &Cell<u16>,
    ) -> HashMap<String, Rect> {
        let (columns, width, height) = self.layout();
        let mut drawn = HashMap::new();
        let mut graph = Buffer::empty(Rect::new(0, 0, width.max(1), height.max(1)));
        for (n, column) in columns.iter().enumerate() {
            self.column(column, n > 0, &mut graph, &mut drawn);
        }
        self.header(area, buf);
        let region = Rect::new(
            area.x,
            area.y + 2,
            area.width,
            area.height.saturating_sub(2 + self.below()),
        );
        let step = CARD + GAP;
        let fit = ((region.width + GAP) / step).max(1);
        let first = self.first(&drawn, columns.len() as u16, fit, offset.get());
        offset.set(first);
        let left = first * step;
        let room = (fit * step).saturating_sub(GAP).min(region.width);
        for y in 0..height.min(region.height) {
            for x in 0..width.saturating_sub(left).min(room) {
                buf[(region.x + x, region.y + y)] = graph[(left + x, y)].clone();
            }
        }
        let dim = Style::new().fg(Color::DarkGray);
        let after = (columns.len() as u16).saturating_sub(first + fit);
        if first > 0 {
            buf.set_string(area.x, area.y + 1, format!("◀ {first} more"), dim);
        }
        if after > 0 {
            let more = format!("{after} more ▶");
            let x = area.right().saturating_sub(more.chars().count() as u16);
            buf.set_string(x, area.y + 1, more, dim);
        }
        let mut areas = HashMap::new();
        for (id, at) in drawn {
            let (from, to) = (at.x.max(left), (at.x + at.width).min(left + room));
            if from < to {
                let moved = Rect::new(
                    region.x + from - left,
                    region.y + at.y,
                    to - from,
                    at.height,
                );
                areas.insert(id, moved.intersection(region));
            }
        }
        let rules = self.rules();
        let top = area.bottom().saturating_sub(rules.len() as u16);
        for (n, (id, text)) in rules.into_iter().enumerate() {
            let y = top + n as u16;
            buf.set_string(area.x, y, "§", Style::new().fg(Color::Magenta));
            let text = cut(text, usize::from(area.width.saturating_sub(2)));
            buf.set_string(area.x + 2, y, text, Style::new());
            areas.insert(id.to_string(), Rect::new(area.x, y, area.width, 1));
        }
        areas
    }

    /// The cards of the graph, a column for each step right under the trail with its card first; the
    /// width and height of the graph.
    fn layout(&self) -> (Vec<Vec<Card<'a>>>, u16, u16) {
        let replaced = self.replaced();
        let parts = self.shown(Trail::new(self.head).parts());
        let inner = usize::from(CARD) - 6;
        let titles: Vec<Vec<String>> = parts
            .iter()
            .map(|(s, _)| title(s.title(), inner, LINES))
            .collect();
        let lines = titles.iter().map(Vec::len).max().unwrap_or(1) as u16;
        let bar = parts
            .iter()
            .any(|(s, _)| !self.shown(s.children()).is_empty());
        let top = 2 + lines + u16::from(bar);
        let mut height = 0;
        let columns: Vec<Vec<Card>> = parts
            .into_iter()
            .zip(titles)
            .enumerate()
            .map(|(n, ((step, state), heading))| {
                let x = n as u16 * (CARD + GAP);
                let under = self.shown(step.tree().into_iter().skip(1).collect());
                let done = under.iter().filter(|(_, s)| *s == State::Done).count();
                let mut column = vec![Card {
                    step,
                    state,
                    replaced: replaced.contains(step.id()),
                    area: Rect::new(x, 0, CARD, top),
                    title: heading,
                    rows: Vec::new(),
                    more: 0,
                    progress: (!under.is_empty()).then_some((done, under.len())),
                }];
                let mut y = top;
                for (child, state) in self.shown(step.children()) {
                    let title = title(child.title(), inner - 3, 2);
                    let mut rows = self.shown(child.children());
                    let more = rows.len().saturating_sub(ROWS);
                    rows.truncate(ROWS);
                    let rows_height = rows.len() + usize::from(more > 0);
                    let tall = 2 + (title.len() + rows_height) as u16;
                    column.push(Card {
                        step: child,
                        state,
                        replaced: replaced.contains(child.id()),
                        area: Rect::new(x + 3, y, CARD - 3, tall),
                        title,
                        rows,
                        more,
                        progress: None,
                    });
                    y += tall;
                }
                height = height.max(y);
                column
            })
            .collect();
        let width = (columns.len() as u16 * (CARD + GAP)).saturating_sub(GAP);
        (columns, width, height)
    }

    /// Draws the cards of `column` on `graph`, the edge into its first card when `joined`, and the line
    /// from its first card to each card under it; adds the area of each step and edge to `drawn`.
    fn column(
        &self,
        column: &[Card],
        joined: bool,
        graph: &mut Buffer,
        drawn: &mut HashMap<String, Rect>,
    ) {
        let first = &column[0];
        let dim = Style::new().fg(Color::DarkGray);
        if joined {
            let (x, y) = (first.area.x - GAP, first.area.y + first.area.height / 2);
            let (line, colour) = match (first.state, first.replaced) {
                (State::Done, false) => ("━", Color::Green),
                (State::Working, false) => ("━", Color::Cyan),
                _ => ("┄", Color::DarkGray),
            };
            let edge = format!("{}▶", line.repeat(usize::from(GAP) - 1));
            graph.set_string(x, y, edge, Style::new().fg(colour));
            drawn.insert(format!("edge:{}", first.step.id()), Rect::new(x, y, GAP, 1));
        }
        for card in column {
            self.card(card, graph, drawn);
        }
        let Some(last) = column.last().filter(|_| column.len() > 1) else {
            return;
        };
        let x = first.area.x + 1;
        graph.set_string(x, first.area.bottom() - 1, "┬", dim);
        let joins: HashSet<u16> = column[1..].iter().map(|c| c.area.y + 1).collect();
        for y in first.area.bottom()..=last.area.y + 1 {
            let line = match (joins.contains(&y), y == last.area.y + 1) {
                (true, true) => "└─",
                (true, false) => "├─",
                _ => "│",
            };
            graph.set_string(x, y, line, dim);
        }
    }

    /// Draws `card` on `graph`: its border, its mark and title, its progress or its rows; adds the area
    /// of its step and of each row's step to `drawn`.
    fn card(&self, card: &Card, graph: &mut Buffer, drawn: &mut HashMap<String, Rect>) {
        let area = card.area;
        let lit = self.selected == Some(card.step.id());
        let (border, colour) = match (card.state, card.replaced) {
            _ if lit => (BorderType::Double, Color::White),
            (_, true) | (State::Dropped, _) => (BorderType::Rounded, Color::DarkGray),
            (State::Open, _) => (BorderType::LightDoubleDashed, Color::DarkGray),
            (State::Working, _) if self.spin.is_some_and(|s| s / 5 % 2 == 1) => {
                (BorderType::Thick, Color::LightCyan)
            }
            (State::Working, _) => (BorderType::Thick, Color::Cyan),
            (State::Done, _) => (BorderType::Rounded, Color::Green),
            (State::Expired, _) => (BorderType::Rounded, Color::Yellow),
        };
        Block::bordered()
            .border_type(border)
            .border_style(Style::new().fg(colour).add_modifier(match lit {
                true => Modifier::BOLD,
                false => Modifier::empty(),
            }))
            .render(area, graph);
        let x = area.x + 2;
        let (mark, tone) = self.look(card.state, card.replaced);
        for (n, line) in card.title.iter().enumerate() {
            let y = area.y + 1 + n as u16;
            if n == 0 {
                graph.set_string(x, y, mark.0, mark.1);
            }
            graph.set_string(x + 2, y, line, tone);
        }
        let room = usize::from(area.width.saturating_sub(6));
        let mut y = area.y + 1 + card.title.len() as u16;
        for (step, state) in &card.rows {
            let (mark, mut tone) = self.look(*state, false);
            if self.selected == Some(step.id()) {
                tone = tone.add_modifier(Modifier::REVERSED);
            }
            graph.set_string(x, y, mark.0, mark.1);
            graph.set_string(x + 2, y, cut(step.title(), room), tone);
            drawn.insert(step.id().to_string(), Rect::new(x, y, area.width - 4, 1));
            y += 1;
        }
        if card.more > 0 {
            let more = format!("+{} more", card.more);
            graph.set_string(x, y, more, Style::new().fg(Color::DarkGray));
        }
        if let Some((done, all)) = card.progress {
            let y = area.bottom() - 2;
            let filled = done * 10 / all;
            graph.set_string(x, y, "▰".repeat(filled), Style::new().fg(Color::Green));
            let rest = "▱".repeat(10 - filled);
            let dim = Style::new().fg(Color::DarkGray);
            graph.set_string(x + filled as u16, y, rest, dim);
            graph.set_string(x + 11, y, format!("{done}/{all}"), dim);
        }
        drawn.insert(card.step.id().to_string(), area);
    }

    /// Draws the trail's mark and title on the first row of `area`, with its project and the done and
    /// all steps shown at the right.
    fn header(&self, area: Rect, buf: &mut Buffer) {
        let steps = self.shown(self.head.tree().into_iter().skip(1).collect());
        let done = steps.iter().filter(|(_, s)| *s == State::Done).count();
        let right = format!(
            "{} · {done}/{}",
            self.head.record().anchor.name,
            steps.len()
        );
        let room = usize::from(area.width).saturating_sub(right.chars().count() + 4);
        buf.set_string(area.x, area.y, "◆", Style::new().fg(Color::Magenta));
        let bold = Style::new().add_modifier(Modifier::BOLD);
        buf.set_string(area.x + 2, area.y, cut(self.head.title(), room), bold);
        let x = area.right().saturating_sub(right.chars().count() as u16);
        buf.set_string(x, area.y, right, Style::new().fg(Color::DarkGray));
    }

    /// The first of `all` columns to show when `fit` fit: `offset`, moved just enough to show the
    /// column of the selected step, else of the newest event's step.
    fn first(&self, drawn: &HashMap<String, Rect>, all: u16, fit: u16, offset: u16) -> u16 {
        let mut first = offset.min(all.saturating_sub(fit));
        let newest = self.history.events[..self.shown]
            .last()
            .map(|e| e.id.as_str());
        if let Some(at) = self.selected.or(newest).and_then(|id| drawn.get(id)) {
            let column = at.x / (CARD + GAP);
            first = first.min(column).max((column + 1).saturating_sub(fit));
        }
        first
    }

    /// The mark of a step in `state`, `replaced` or not, with its style, and the style of its title.
    fn look(&self, state: State, replaced: bool) -> ((&'static str, Style), Style) {
        let dim = Style::new().fg(Color::DarkGray);
        let gone = dim.add_modifier(Modifier::CROSSED_OUT);
        match (state, replaced) {
            (_, true) => (("↺", dim), gone),
            (State::Open, _) => (("○", dim), dim),
            (State::Working, _) => {
                let mark = self.spin.map_or("◐", |s| SPINNER[s % SPINNER.len()]);
                let bold = Style::new().add_modifier(Modifier::BOLD);
                ((mark, Style::new().fg(Color::Cyan)), bold)
            }
            (State::Done, _) => (("✓", Style::new().fg(Color::Green)), Style::new()),
            (State::Dropped, _) => (("✗", Style::new().fg(Color::Red)), gone),
            (State::Expired, _) => {
                let yellow = Style::new().fg(Color::Yellow);
                (("◌", yellow), yellow)
            }
        }
    }

    /// The rows the rules take under the graph: a blank one and one for each rule shown, none with no
    /// rule.
    fn below(&self) -> u16 {
        match self.rules().len() {
            0 => 0,
            n => 1 + n as u16,
        }
    }

    /// The id and text of the newest rules shown, oldest first.
    fn rules(&self) -> Vec<(&str, &str)> {
        let shown = self.history.events[..self.shown].iter();
        let rules: Vec<(&str, &str)> = shown
            .filter(|e| e.kind == Kind::Rule)
            .map(|e| (e.id.as_str(), e.text.as_str()))
            .collect();
        rules[rules.len().saturating_sub(RULES)..].to_vec()
    }

    /// The ids of the steps a step shown replaces.
    fn replaced(&self) -> HashSet<String> {
        let shown = self.shown(self.head.tree());
        let replaces = shown
            .iter()
            .filter_map(|(s, _)| s.record().replaces.as_ref());
        replaces.map(|r| r.to_string()).collect()
    }

    /// Each of `steps` that is shown, with where it stands.
    fn shown(&self, steps: Vec<Step<'a>>) -> Vec<(Step<'a>, State)> {
        let state = |s: Step<'a>| Some((s, self.history.state(s.id(), self.shown)?));
        steps.into_iter().filter_map(state).collect()
    }
}

/// `text` wrapped to `width`, at most `most` rows, the last cut with `…` when it had more.
fn title(text: &str, width: usize, most: usize) -> Vec<String> {
    let mut lines = wrap(text, width);
    if lines.len() > most {
        lines.truncate(most);
        let last = lines.pop().unwrap_or_default();
        lines.push(cut(&format!("{last} …"), width));
    }
    lines
}
