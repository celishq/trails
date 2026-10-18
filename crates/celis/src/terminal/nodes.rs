//! `celis tree --graph`: the node view, one trail as a graph, live or played back event by event.

use super::canvas::rows;
use super::flow::Flow;
use super::history::{Event, History, Kind};
use super::live::Live;
use super::screen::{spin, Screen, TURN};
use super::wake::{Key, Wake};
use crate::command::tree::{cut, today, EMPTY};
use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::progress::Model;
use celis_nostr::Note;
use celis_trails::{Mark, State, Step};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Widget, Wrap};
use std::cell::Cell;
use std::collections::HashMap;
use std::io::{IsTerminal, Write};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};
use tachyonfx::{fx, Effect, Interpolation, Motion};

/// The time between two events as the view plays them.
const BEAT: Duration = Duration::from_millis(900);

/// The time between two frames while an effect plays.
const FRAME: Duration = Duration::from_millis(30);

/// The keys the node view reads, for its last row.
const KEYS: &str =
    "arrows select · enter details · [ ] event · space play · g G first last · tab trail · q quit";

impl Command {
    /// Draws the trail of the step `args` names, else the newest trail of today, as a graph: on a
    /// terminal live on the alternate screen and moved through its events by keys, else printed once
    /// as it stands.
    pub(crate) fn nodes(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let id = args.words.get(1);
        if let (false, Some(id)) = (self.held()?, id) {
            return Err(Failure::no_step(id));
        }
        if std::io::stdout().is_terminal() && !args.json {
            return self.screen(out, |out, wakes| {
                let mut screen = Screen::default();
                let empty = format!("{EMPTY}\n");
                if !self.waited(&mut screen, out, (&empty, KEYS), &wakes.1)? {
                    return Ok(());
                }
                let trails = self.trails()?;
                if let Some(id) = id {
                    trails.steps().find(id)?;
                }
                let mut nodes = Nodes::new(self, id.cloned());
                self.live(&mut nodes, trails, &mut screen, out, wakes)
            });
        }
        let text = match self.held()? {
            true => Model::of(&self.trails()?, |model| printed(model, id))?,
            false => format!("{EMPTY}\n"),
        };
        write!(out, "{text}").map_err(anyhow::Error::from)?;
        Ok(())
    }
}

/// The graph of the trail of `id`, else of the newest trail of today, as it stands, as plain text;
/// `EMPTY` with no trail.
fn printed(model: &Model, id: Option<&String>) -> Result<String, Failure> {
    let head = match id {
        Some(id) => model.steps.find(id)?.trail(),
        None => match today(model.steps).first() {
            Some(head) => *head,
            None => return Ok(format!("{EMPTY}\n")),
        },
    };
    let history = History::of(head, model.decisions);
    let flow = Flow {
        head,
        history: &history,
        shown: history.events.len(),
        selected: None,
        spin: None,
    };
    let (width, height) = flow.size();
    let mut buf = Buffer::empty(Rect::new(0, 0, width.max(80), height));
    flow.render(buf.area, &mut buf, &Cell::new(0));
    let rows = rows(&buf, false);
    Ok(rows.iter().map(|r| format!("{}\n", r.trim_end())).collect())
}

/// A node view: the trail it shows, the events shown, the step selected and whether its details are
/// open, whether it follows new events, whether it plays and when its next beat is, the effects that
/// play and the events still to start one, the graph's offset, and when it drew last.
struct Nodes<'a> {
    command: &'a Command,
    trail: Option<String>,
    shown: usize,
    selected: Option<String>,
    details: bool,
    live: bool,
    beat: Option<Instant>,
    effects: Vec<Effect>,
    fresh: Vec<usize>,
    offset: Cell<u16>,
    drawn: Instant,
}

impl<'a> Nodes<'a> {
    /// A live node view of the trail of the step `id`, else of the newest trail of today.
    fn new(command: &'a Command, id: Option<String>) -> Nodes<'a> {
        Nodes {
            command,
            trail: id,
            shown: usize::MAX,
            selected: None,
            details: false,
            live: true,
            beat: None,
            effects: Vec::new(),
            fresh: Vec::new(),
            offset: Cell::new(0),
            drawn: Instant::now(),
        }
    }

    /// The trail it shows in `model`: of the step it names, else the newest of today, which it then
    /// names.
    fn head<'m>(&mut self, model: &'m Model) -> Option<Step<'m>> {
        let head = match &self.trail {
            Some(id) => model.steps.find(id).ok()?.trail(),
            None => *today(model.steps).first()?,
        };
        self.trail = Some(head.id().to_string());
        Some(head)
    }

    /// Moves to the event `to` of `history`, with an effect for each event it passes on the way
    /// forward and none left on the way back.
    fn go(&mut self, to: usize, history: &History) {
        let to = to.min(history.events.len());
        match to > self.shown {
            true => self.fresh.extend(self.shown..to),
            false => {
                self.effects.clear();
                self.fresh.clear();
            }
        }
        self.shown = to;
        self.live = to == history.events.len();
    }

    /// The bottom two rows of `area` on `buf`: whether it plays, follows or stands at an event, with the
    /// newest event shown, then a bar of the events shown.
    fn timeline(&self, history: &History, area: Rect, buf: &mut Buffer) {
        let all = history.events.len();
        let (status, colour) = match (self.beat.is_some(), self.live) {
            (true, _) => (format!("▶ {}/{all}", self.shown), Color::Cyan),
            (false, true) => ("● live".to_string(), Color::Green),
            (false, false) => (format!("⏸ {}/{all}", self.shown), Color::Yellow),
        };
        let style = Style::new().fg(colour).add_modifier(Modifier::BOLD);
        let width = usize::from(area.width);
        buf.set_string(area.x, area.y, &status, style);
        if let Some(event) = history.events[..self.shown].last() {
            let said = format!("{} {}", word(event.kind), event.text);
            let room = width.saturating_sub(status.chars().count() + 3);
            let x = area.x + status.chars().count() as u16 + 3;
            buf.set_string(x, area.y, cut(&said, room), Style::new());
        }
        let at = match all {
            0 => 0,
            all => self.shown * width.saturating_sub(1) / all,
        };
        for x in 0..width {
            let (bar, colour) = match x.cmp(&at) {
                std::cmp::Ordering::Less => ("━", Color::Cyan),
                std::cmp::Ordering::Equal => ("●", Color::White),
                std::cmp::Ordering::Greater => ("─", Color::DarkGray),
            };
            buf.set_string(area.x + x as u16, area.y + 1, bar, Style::new().fg(colour));
        }
    }
}

impl Live for Nodes<'_> {
    fn renew(&mut self, _: &Model, _: Option<&[Note]>) {}

    /// Draws the trail of `model` as it stood after the events shown, then again each frame while an
    /// effect plays, each beat while it plays, and each turn of the spinner, until a wake comes; a
    /// live view first moves to the newest event.
    fn wait(
        &mut self,
        model: &Model,
        screen: &mut Screen,
        out: &mut dyn Write,
        events: &Receiver<Wake>,
    ) -> Result<Result<Wake, RecvTimeoutError>, Failure> {
        loop {
            let Some(head) = self.head(model) else {
                screen.draw(out, &format!("{EMPTY}\n"), None, KEYS)?;
                return Ok(events.recv_timeout(Duration::MAX));
            };
            let history = History::of(head, model.decisions);
            let all = history.events.len();
            match self.shown == usize::MAX {
                true => self.shown = all,
                false if self.live => self.go(all, &history),
                false => self.shown = self.shown.min(all),
            }
            let now = Instant::now();
            if let Some(beat) = self.beat.filter(|b| now >= *b) {
                self.go(self.shown + 1, &history);
                self.beat = (!self.live).then_some(beat + BEAT);
            }
            let size = crate::command::size();
            let width = size.map_or(80, |s| s.ws_col.saturating_sub(1)).max(20);
            let height = size.map_or(24, |s| s.ws_row.saturating_sub(1)).max(6);
            let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
            let flow = Flow {
                head,
                history: &history,
                shown: self.shown,
                selected: None,
                spin: Some(spin()),
            };
            let columns = flow.columns();
            if !columns
                .iter()
                .flatten()
                .any(|id| Some(id) == self.selected.as_ref())
            {
                (self.selected, self.details) = (None, false);
            }
            let flow = Flow {
                selected: self.selected.as_deref(),
                ..flow
            };
            let graph = Rect::new(0, 0, width, height - 3);
            let areas = flow.render(graph, &mut buf, &self.offset);
            self.timeline(&history, Rect::new(0, height - 2, width, 2), &mut buf);
            let colour = self.command.colour();
            let passed = now.duration_since(self.drawn);
            self.drawn = now;
            for effect in &mut self.effects {
                effect.process(passed.into(), &mut buf, graph);
            }
            for n in std::mem::take(&mut self.fresh) {
                let Some(mut effect) = effect(&history.events[n], &areas) else {
                    continue;
                };
                effect.process(Duration::ZERO.into(), &mut buf, graph);
                self.effects.push(effect);
            }
            self.effects.retain(|e| e.running() && colour);
            if let Some(step) = self.selected.as_ref().filter(|_| self.details) {
                let step = model.steps.find(step).ok();
                let state = step.and_then(|s| history.state(s.id(), self.shown));
                if let (Some(step), Some(state)) = (step, state) {
                    details(step, state, graph, &mut buf);
                }
            }
            let text = rows(&buf, colour).join("\n");
            screen.draw(out, &text, None, KEYS)?;
            let working = (0..self.shown).any(|n| {
                let event = &history.events[n];
                history.state(&event.id, self.shown) == Some(State::Working)
            });
            let waits = [
                (!self.effects.is_empty()).then_some(FRAME),
                self.beat
                    .map(|b| b.saturating_duration_since(Instant::now())),
                working.then_some(TURN),
            ];
            let wait = waits.into_iter().flatten().min().unwrap_or(Duration::MAX);
            match events.recv_timeout(wait) {
                Err(RecvTimeoutError::Timeout) => {}
                wake => return Ok(wake),
            }
        }
    }

    /// Selects the card or row next to the selected one, opens or closes its details, moves an event
    /// back or forth, to the first or the newest, plays or stops, or shows the trail before or after
    /// it of today.
    fn press(&mut self, key: Key, model: &Model, _: &mut Screen) -> Result<(), Failure> {
        let Some(head) = self.head(model) else {
            return Ok(());
        };
        let history = History::of(head, model.decisions);
        let all = history.events.len();
        match key {
            Key::Up | Key::Down | Key::Left | Key::Right => {
                let flow = Flow {
                    head,
                    history: &history,
                    shown: self.shown.min(all),
                    selected: None,
                    spin: None,
                };
                let newest = history.events[..self.shown.min(all)].last();
                let from = self.selected.clone().or(newest.map(|e| e.id.clone()));
                self.selected = next(&flow.columns(), from.as_deref(), key);
            }
            Key::Enter => self.details = !self.details && self.selected.is_some(),
            Key::Escape => self.details = false,
            Key::Back => {
                self.beat = None;
                self.go(self.shown.saturating_sub(1), &history);
            }
            Key::Forth => {
                self.beat = None;
                self.go(self.shown + 1, &history);
            }
            Key::Top => {
                self.beat = None;
                self.go(0, &history);
            }
            Key::End => {
                self.beat = None;
                self.go(all, &history);
            }
            Key::Space if self.beat.is_some() => self.beat = None,
            Key::Space => {
                if self.shown == all {
                    self.go(0, &history);
                }
                self.live = false;
                self.beat = Some(Instant::now() + BEAT);
            }
            Key::Tab | Key::BackTab => {
                let trails = today(model.steps);
                let at = trails.iter().position(|t| t.id() == head.id());
                let to = match (key, at) {
                    (Key::BackTab, Some(at)) => at.checked_sub(1),
                    (_, Some(at)) => Some(at + 1),
                    _ => Some(0),
                };
                if let Some(trail) = to.and_then(|to| trails.get(to)) {
                    self.trail = Some(trail.id().to_string());
                    (self.shown, self.live, self.beat) = (usize::MAX, true, None);
                    (self.selected, self.details) = (None, false);
                    self.effects.clear();
                    self.offset.set(0);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// The step to select in `columns` after `key` from the step `from`: the one above or below it in its
/// column, or at its height in the column before or after; the first card when `from` is in none.
fn next(columns: &[Vec<String>], from: Option<&str>, key: Key) -> Option<String> {
    let at = columns.iter().enumerate().find_map(|(c, column)| {
        let row = column.iter().position(|id| Some(id.as_str()) == from)?;
        Some((c, row))
    });
    let Some((c, row)) = at else {
        return columns.first()?.first().cloned();
    };
    let (c, row) = match key {
        Key::Up => (c, row.saturating_sub(1)),
        Key::Down => (c, row + 1),
        Key::Left => (c.saturating_sub(1), row),
        _ => ((c + 1).min(columns.len() - 1), row),
    };
    let column = &columns[c];
    column.get(row.min(column.len() - 1)).cloned()
}

/// Draws the details of `step` as it stands, `state`, over `area` of `buf`: its whole text, where it
/// works and its check, how many of its steps are done, and its proof or why it was dropped.
fn details(step: Step, state: State, area: Rect, buf: &mut Buffer) {
    let area = Rect::new(
        area.x + 2,
        area.y + 1,
        area.width.saturating_sub(4),
        area.height.saturating_sub(1),
    );
    let dim = Style::new().fg(Color::DarkGray);
    let bold = Style::new().add_modifier(Modifier::BOLD);
    let field = |name: &str, value: String, style: Style| {
        Line::from(vec![
            Span::styled(format!("{name:<8}"), dim),
            Span::styled(value, style),
        ])
    };
    let record = step.record();
    let mut lines: Vec<Line> = record
        .content
        .lines()
        .enumerate()
        .map(|(n, line)| match n {
            0 => Line::styled(line.to_string(), bold),
            _ => Line::raw(line.to_string()),
        })
        .collect();
    lines.push(Line::raw(""));
    let (said, colour) = match state {
        State::Open => ("open", Color::DarkGray),
        State::Working => ("working", Color::Cyan),
        State::Done => ("done", Color::Green),
        State::Dropped => ("dropped", Color::Red),
        State::Expired => ("expired", Color::Yellow),
    };
    lines.push(field("state", said.to_string(), Style::new().fg(colour)));
    if !record.places.is_empty() {
        lines.push(field("at", record.places.join(", "), Style::new()));
    }
    if let Some(check) = &record.check {
        lines.push(field("check", check.clone(), Style::new()));
    }
    let under = step.tree().len() - 1;
    if under > 0 {
        let done = step.tree()[1..]
            .iter()
            .filter(|s| s.state() == State::Done)
            .count();
        lines.push(field("steps", format!("{done}/{under} done"), Style::new()));
    }
    match step.close().map(|(_, mark)| mark) {
        Some(Mark::Done { proof, changed }) if state == State::Done => {
            lines.push(field("proof", proof, Style::new().fg(Color::Green)));
            if let Some(changed) = changed {
                lines.push(field("changed", changed, Style::new().fg(Color::Yellow)));
            }
        }
        Some(Mark::Drop { reason }) if state == State::Dropped => {
            lines.push(field("why", reason, Style::new().fg(Color::Red)));
        }
        _ => {}
    }
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(Color::White))
        .title(format!(" {} ", &step.id()[..8]))
        .title_bottom(Line::styled(" enter esc close ", dim).right_aligned());
    Clear.render(area, buf);
    Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .render(area, buf);
}

/// The effect of `event` on the area it drew in `areas`: a step added comes together, a started one
/// sweeps in with its edge, a done one flashes white and a dropped one red, and a rule slides in.
fn effect(event: &Event, areas: &HashMap<String, Rect>) -> Option<Effect> {
    let area = *areas.get(&event.id)?;
    let edge = areas.get(&format!("edge:{}", event.id)).copied();
    let timer = |ms| (ms, Interpolation::QuadOut);
    let on = |effect: Effect, at: Rect| effect.with_area(at);
    let effect = match event.kind {
        Kind::Added => on(fx::coalesce(timer(500)), area),
        Kind::Started => {
            let sweep = |at| {
                on(
                    fx::sweep_in(Motion::LeftToRight, 8, 0, Color::Black, timer(600)),
                    at,
                )
            };
            let mut both = vec![sweep(area)];
            both.extend(edge.map(sweep));
            fx::parallel(&both)
        }
        Kind::Done => {
            let flash = |at| on(fx::fade_from_fg(Color::White, timer(900)), at);
            let mut both = vec![flash(area)];
            both.extend(edge.map(flash));
            fx::parallel(&both)
        }
        Kind::Dropped => on(fx::fade_from_fg(Color::Red, timer(900)), area),
        Kind::Rule => on(
            fx::slide_in(Motion::LeftToRight, 12, 0, Color::Black, timer(700)),
            area,
        ),
    };
    Some(effect)
}

/// The word the timeline says for an event of `kind`.
fn word(kind: Kind) -> &'static str {
    match kind {
        Kind::Added => "added",
        Kind::Rule => "§ rule",
        Kind::Started => "started",
        Kind::Done => "done",
        Kind::Dropped => "dropped",
    }
}

#[cfg(test)]
mod tests;
