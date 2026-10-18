//! `celis tree --graph`: the node view, one trail as a graph, live or played back event by event.

use crate::command::tree::{today, EMPTY};
use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::progress::Model;
use crate::terminal::canvas::{lines, Styled};
use crate::terminal::live::{Live, Page};
use crate::terminal::look::{spin, TURN};
use crate::terminal::screen::Screen;
use crate::terminal::wake::Key;
use celis_nostr::Note;
use celis_trails::{State, Step};
use details::details;
use diagram::Diagram;
use effect::effect;
use events::Events;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::cell::Cell;
use std::io::{IsTerminal, Write};
use std::time::{Duration, Instant};
use tachyonfx::Effect;
use timeline::timeline;

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
    pub(crate) fn graph(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let id = args.words.get(1);
        if let (false, Some(id)) = (self.held()?, id) {
            return Err(Failure::no_step(id));
        }
        if std::io::stdout().is_terminal() && !args.json {
            return self.live(out, empty(), |trails| {
                if let Some(id) = id {
                    trails.steps().find(id)?;
                }
                Ok(Replay::new(self, id.cloned()))
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
    let events = Events::of(head, model.decisions);
    let diagram = Diagram {
        head,
        events: &events,
        shown: events.all.len(),
        selected: None,
        spin: None,
    };
    let (width, height) = diagram.size();
    let mut buf = Buffer::empty(Rect::new(0, 0, width.max(80), height));
    diagram.render(buf.area, &mut buf, &Cell::new(0));
    let rows = lines(&buf);
    Ok(rows
        .iter()
        .map(|r| format!("{}\n", r.to_string().trim_end()))
        .collect())
}

/// A node view: the trail it shows, the events shown, the step selected and whether its details are
/// open, whether it follows new events, whether it plays and when its next beat is, the effects that
/// play and the events still to start one, the graph's offset, and when it drew last.
struct Replay<'a> {
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

impl<'a> Replay<'a> {
    /// A live node view of the trail of the step `id`, else of the newest trail of today.
    fn new(command: &'a Command, id: Option<String>) -> Replay<'a> {
        Replay {
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

    /// Moves to the event `to` of `events`, with an effect for each event it passes on the way
    /// forward and none left on the way back.
    fn go(&mut self, to: usize, events: &Events) {
        let to = to.min(events.all.len());
        match to > self.shown {
            true => self.fresh.extend(self.shown..to),
            false => {
                self.effects.clear();
                self.fresh.clear();
            }
        }
        self.shown = to;
        self.live = to == events.all.len();
    }
}

impl Live for Replay<'_> {
    fn renew(&mut self, _: &Model, _: Option<&[Note]>) {}

    /// The trail of `model` as it stood after the events shown, drawn again each frame while an effect
    /// plays, each beat while it plays, and each turn of the spinner; a live view first moves to the
    /// newest event.
    fn page(&mut self, model: &Model, _: &mut Screen) -> Result<Page, Failure> {
        let Some(head) = self.head(model) else {
            return Ok(empty());
        };
        let events = Events::of(head, model.decisions);
        let all = events.all.len();
        match self.shown == usize::MAX {
            true => self.shown = all,
            false if self.live => self.go(all, &events),
            false => self.shown = self.shown.min(all),
        }
        let now = Instant::now();
        if let Some(beat) = self.beat.filter(|b| now >= *b) {
            self.go(self.shown + 1, &events);
            self.beat = (!self.live).then_some(beat + BEAT);
        }
        let size = crate::command::size();
        let width = size.map_or(80, |s| s.ws_col.saturating_sub(1)).max(20);
        let height = size.map_or(24, |s| s.ws_row.saturating_sub(1)).max(6);
        let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
        let diagram = Diagram {
            head,
            events: &events,
            shown: self.shown,
            selected: None,
            spin: Some(spin()),
        };
        let columns = diagram.columns();
        if !columns
            .iter()
            .flatten()
            .any(|id| Some(id) == self.selected.as_ref())
        {
            (self.selected, self.details) = (None, false);
        }
        let diagram = Diagram {
            selected: self.selected.as_deref(),
            ..diagram
        };
        let graph = Rect::new(0, 0, width, height - 3);
        let areas = diagram.render(graph, &mut buf, &self.offset);
        let playing = (self.beat.is_some(), self.live);
        timeline(
            &events,
            self.shown,
            playing,
            Rect::new(0, height - 2, width, 2),
            &mut buf,
        );
        let colour = self.command.colour();
        let passed = now.duration_since(self.drawn);
        self.drawn = now;
        for effect in &mut self.effects {
            effect.process(passed.into(), &mut buf, graph);
        }
        for n in std::mem::take(&mut self.fresh) {
            let Some(mut effect) = effect(&events.all[n], &areas) else {
                continue;
            };
            effect.process(Duration::ZERO.into(), &mut buf, graph);
            self.effects.push(effect);
        }
        self.effects.retain(|e| e.running() && colour);
        if let Some(step) = self.selected.as_ref().filter(|_| self.details) {
            let step = model.steps.find(step).ok();
            let state = step.and_then(|s| events.state(s.id(), self.shown));
            if let (Some(step), Some(state)) = (step, state) {
                details(step, state, graph, &mut buf);
            }
        }
        let working = (0..self.shown).any(|n| {
            let event = &events.all[n];
            events.state(&event.id, self.shown) == Some(State::Working)
        });
        let waits = [
            (!self.effects.is_empty()).then_some(FRAME),
            self.beat
                .map(|b| b.saturating_duration_since(Instant::now())),
            working.then_some(TURN),
        ];
        Ok(Page {
            lines: lines(&buf),
            keys: KEYS.into(),
            again: waits.into_iter().flatten().min(),
            ..Page::default()
        })
    }

    /// Selects the card or row next to the selected one, opens or closes its details, moves an event
    /// back or forth, to the first or the newest, plays or stops, or shows the trail before or after
    /// it of today.
    fn press(&mut self, key: Key, model: &Model, _: &mut Screen) -> Result<(), Failure> {
        let Some(head) = self.head(model) else {
            return Ok(());
        };
        let events = Events::of(head, model.decisions);
        let all = events.all.len();
        match key {
            Key::Up | Key::Down | Key::Left | Key::Right => {
                let diagram = Diagram {
                    head,
                    events: &events,
                    shown: self.shown.min(all),
                    selected: None,
                    spin: None,
                };
                let newest = events.all[..self.shown.min(all)].last();
                let from = self.selected.clone().or(newest.map(|e| e.id.clone()));
                self.selected = next(&diagram.columns(), from.as_deref(), key);
            }
            Key::Enter => self.details = !self.details && self.selected.is_some(),
            Key::Escape => self.details = false,
            Key::Back => {
                self.beat = None;
                self.go(self.shown.saturating_sub(1), &events);
            }
            Key::Forth => {
                self.beat = None;
                self.go(self.shown + 1, &events);
            }
            Key::Top => {
                self.beat = None;
                self.go(0, &events);
            }
            Key::End => {
                self.beat = None;
                self.go(all, &events);
            }
            Key::Space if self.beat.is_some() => self.beat = None,
            Key::Space => {
                if self.shown == all {
                    self.go(0, &events);
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

/// The page of no trail.
fn empty() -> Page {
    Page {
        lines: vec![Styled::from(EMPTY)],
        keys: KEYS.into(),
        ..Page::default()
    }
}

mod details;
mod diagram;
mod effect;
mod events;
#[cfg(test)]
mod tests;
mod timeline;
