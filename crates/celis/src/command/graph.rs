//! `celis tree --graph`: every trail of today under its project's folder, and each trail as a graph of
//! cards, live.

use crate::command::tree::EMPTY;
use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::progress::{Grade, Model};
use crate::terminal::bar::bar;
use crate::terminal::canvas::{cut, lines, paint, Styled, BOLD, CYAN, DIM, FRAME, GREEN};
use crate::terminal::live::{Live, Page};
use crate::terminal::look::lane;
use crate::terminal::screen::Screen;
use crate::terminal::unfold::{ease, mix, toward};
use crate::terminal::wake::Key;
use crate::text::clock::took;
use card::{inside, put, Card};
use celis_nostr::Note;
use celis_trails::{State, Step, Trail};
use folders::folders;
use layout::{breadth, columns, Layout, GAP};
use moment::{tally, Moment, LIGHT};
use overview::{Overview, TALL};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;
use rules::{Learned, Rules};
use seen::Seen;
use std::io::{IsTerminal, Write};
use std::time::{Duration, Instant};
use view::{Mode, View};

/// The rows the head of the view takes: its path, a line, its counts, and a gap.
const HEAD: u16 = 4;

/// The most rows the rules card pinned under the graph takes.
const PINNED: u16 = 8;

/// The width a graph prints at off a terminal.
const PRINTED: u16 = 108;

impl Command {
    /// Draws every trail of today, or the trail of the step `args` names: on a terminal live on the
    /// alternate screen and moved through by keys, else printed once as it stands.
    pub(crate) fn graph(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let id = args.words.get(1).cloned();
        if let (false, Some(id)) = (self.held()?, &id) {
            return Err(Failure::no_step(id));
        }
        if std::io::stdout().is_terminal() && !args.json {
            return self.live(out, empty(), |trails| {
                let trail = match &id {
                    Some(id) => Some(trails.steps().find(id)?.trail().id().to_string()),
                    None => None,
                };
                Ok(Graph::new(trail))
            });
        }
        let text = match self.held()? {
            true => Model::of(&self.trails()?, |model| printed(model, id.as_deref()))?,
            false => format!("{EMPTY}\n"),
        };
        write!(out, "{text}").map_err(anyhow::Error::from)?;
        Ok(())
    }
}

/// The live graph view: the trail it was asked for, where the person is, and what it saw change.
struct Graph {
    named: Option<String>,
    view: Option<View>,
    seen: Seen,
}

impl Graph {
    fn new(named: Option<String>) -> Graph {
        Graph {
            named,
            view: None,
            seen: Seen::new(),
        }
    }
}

impl Live for Graph {
    fn renew(&mut self, _: &Model, _: Option<&[Note]>) {}

    /// The screen at this moment, as tall as the terminal under its key line, drawn again each frame
    /// while something moves, each pulse while a card works, and soon otherwise.
    fn page(&mut self, model: &Model, _: &mut Screen) -> Result<Page, Failure> {
        let size = crate::command::size();
        let width = size.map_or(PRINTED, |s| s.ws_col.saturating_sub(1)).max(30);
        let rows = size
            .map_or(40, |s| s.ws_row.saturating_sub(1))
            .max(HEAD + 6);
        let now = Instant::now();
        let trails = folders(model.steps);
        self.seen.note(&trails, |t| rule_ids(model, t), now);
        let view = self
            .view
            .get_or_insert_with(|| View::new(&trails, self.named.as_deref()));
        let lines = frame(model, &trails, view, &mut self.seen, (width, rows), now);
        let lively = self.seen.lively(now) || view.offset.fract() != 0.0;
        let working = trails.iter().any(|t| t.state() == State::Working);
        let again = match (lively, working) {
            (true, _) => Duration::from_millis(16),
            (false, true) => Duration::from_millis(33),
            (false, false) => Duration::from_millis(250),
        };
        Ok(Page {
            lines,
            keys: view.keys().into(),
            again: Some(again),
            ..Page::default()
        })
    }

    fn press(&mut self, key: Key, model: &Model, _: &mut Screen) -> Result<(), Failure> {
        if let Some(view) = self.view.as_mut() {
            view.press(key, model.steps);
        }
        Ok(())
    }

    fn typing(&self) -> bool {
        self.view.as_ref().is_some_and(|v| v.query.is_some())
    }
}

/// The lines of the screen, `width` by `rows`, at `now`: the head, then the rules over the rest when
/// the person opened them; else the part of All or of the trail's graph that fits, with how many rows
/// it hides above and below, and the rules card pinned under it. On All a trail that starts pushes
/// what is below it down, and one that starts above the screen keeps the screen still and is marked
/// on the line above; All scrolls only to show the trail the person picks.
fn frame(
    model: &Model,
    trails: &[Step],
    view: &mut View,
    seen: &mut Seen,
    (width, rows): (u16, u16),
    now: Instant,
) -> Vec<Styled> {
    view.trails = trails.iter().map(|t| t.id().to_string()).collect();
    let matches: Vec<Step> = trails
        .iter()
        .filter(|t| view.finds(t.title(), &t.record().anchor.name))
        .copied()
        .collect();
    view.matches = matches.iter().map(|t| t.id().to_string()).collect();
    if !view
        .matches
        .contains(view.pick.as_ref().unwrap_or(&String::new()))
    {
        view.pick = view.matches.first().cloned();
    }
    let shown = view
        .trail
        .as_deref()
        .and_then(|id| trails.iter().find(|t| t.id() == id))
        .copied();
    if view.mode() == Mode::Trail && shown.is_none() {
        view.trail = trails.first().map(|t| t.id().to_string());
    }
    let shown = view
        .trail
        .as_deref()
        .and_then(|id| trails.iter().find(|t| t.id() == id))
        .copied();
    match (view.mode(), shown) {
        (Mode::Trail, Some(trail)) => seen.show(trail.id(), now),
        _ => trails.iter().for_each(|t| seen.show(t.id(), now)),
    }
    let unfolded = view.unfolded.clone();
    let m = Moment {
        seen,
        now,
        unfolded: &unfolded,
        at: model.steps.now(),
    };
    let ruled: Vec<Step> = match (view.mode(), shown) {
        (Mode::Trail, Some(trail)) => vec![trail],
        _ => matches.clone(),
    };
    let rules = learned(model, &ruled, view.mode() == Mode::All);
    view.learned = rules.learned.iter().map(|r| r.id.clone()).collect();
    let mut out = lines(&head(trails, &matches, shown, view, width, &m));
    if view.rules {
        let body = rules.opened(width, rows - HEAD, view, &m);
        out.extend(lines(&body));
        return out;
    }
    let (canvas, focus) = match (view.mode(), shown) {
        (Mode::Trail, Some(trail)) => chain(trail, width, view, &m),
        _ => {
            let drawn = Overview::draw(&matches, width, view, &m);
            if !view.places.is_empty() {
                for (id, y) in &drawn.places {
                    if !view.places.contains_key(id) && f64::from(*y) < view.offset {
                        view.offset += f64::from(TALL);
                        view.above = Some(now);
                    }
                }
            }
            view.places = drawn.places;
            let focus = drawn.focus.filter(|_| view.reveal);
            (drawn.buf, focus)
        }
    };
    let pinned = rules.pinned(width, PINNED, &m);
    let below = pinned.as_ref().map_or(0, |p| p.area.height + 1);
    let room = rows.saturating_sub(HEAD + 2 + below).max(4);
    let height = canvas.area.height;
    if let Some((top, bottom)) = focus {
        let mut goal = view.offset;
        if f64::from(bottom) - f64::from(room) > goal {
            goal = f64::from(bottom) - f64::from(room) + 1.0;
        }
        if f64::from(top) < goal {
            goal = f64::from(top);
        }
        let goal = goal.clamp(0.0, f64::from(height.saturating_sub(room)));
        view.offset += (goal - view.offset) * 0.22;
        if (goal - view.offset).abs() < 0.5 {
            view.offset = goal;
            view.reveal = false;
        }
    }
    view.offset = view
        .offset
        .clamp(0.0, f64::from(height.saturating_sub(room)));
    let first = view.offset.round() as u16;
    let body = lines(&canvas);
    let fresh = view.mode() == Mode::All
        && view
            .above
            .is_some_and(|a| now.duration_since(a).as_secs() < 6);
    out.push(more("▲", first, width, fresh && first > 0));
    for y in first..first + room {
        out.push(body.get(usize::from(y)).cloned().unwrap_or_default());
    }
    out.push(more("▼", height.saturating_sub(first + room), width, false));
    if let Some(pinned) = pinned {
        out.extend(lines(&pinned));
    }
    out
}

/// The line that says how many rows are hidden above or below, `n`, at the right of `width`; with
/// `fresh`, that a new trail started above. Blank when none are hidden.
fn more(arrow: &str, n: u16, width: u16, fresh: bool) -> Styled {
    if n == 0 {
        return Styled::default();
    }
    let side = if arrow == "▲" { "above" } else { "below" };
    let text = format!("{arrow} {n} rows {side}");
    let news = if fresh {
        "● a new trail above  ·  "
    } else {
        ""
    };
    let pad = usize::from(width).saturating_sub(2 + text.chars().count() + news.chars().count());
    Styled::from(vec![
        Span::raw(" ".repeat(pad)),
        paint(CYAN, news),
        paint(DIM, &text),
    ])
}

/// The head of the screen: on All, `All trails` with the search the person types and how many trails
/// match, a line, and how many trails, projects, active ones, steps at work and finished ones there
/// are; on a trail, its path `All trails › 📁 project › trail` with how many other trails moved since
/// she last saw them, a line, and its steps by state with its bar, done of all and time worked.
fn head(
    trails: &[Step],
    matches: &[Step],
    shown: Option<Step>,
    view: &View,
    width: u16,
    m: &Moment,
) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, width, HEAD));
    let rule = "─".repeat(usize::from(width.saturating_sub(4)));
    buf.set_string(2, 1, rule, FRAME);
    let (Mode::Trail, Some(trail)) = (view.mode(), shown) else {
        let mut x = put(&mut buf, 2, 0, &[paint(BOLD, "All trails")]);
        if let Some(query) = &view.query {
            let found = match matches.len() {
                0 => "no trail matches".to_string(),
                n => format!("{n} of {}", trails.len()),
            };
            let light = Style::new().fg(LIGHT).patch(BOLD);
            x = put(
                &mut buf,
                x + 4,
                0,
                &[paint(light, "/ "), paint(light, query), paint(CYAN, "▏")],
            );
            buf.set_string(x + 2, 0, found, DIM);
        }
        let working: usize = trails
            .iter()
            .map(|t| {
                t.tree()
                    .iter()
                    .filter(|s| s.state() == State::Working)
                    .count()
            })
            .sum();
        let finished = trails.iter().filter(|t| t.state() == State::Done).count();
        let mut projects: Vec<&str> = trails
            .iter()
            .map(|t| t.record().anchor.project.as_str())
            .collect();
        projects.dedup();
        let mut line = vec![
            paint(Style::new(), &format!("{} trails today", trails.len())),
            paint(
                DIM,
                &format!(
                    "  ·  {} projects  ·  {} active  ·  {working} steps at work",
                    projects.len(),
                    trails.len() - finished
                ),
            ),
        ];
        if finished > 0 {
            line.push(paint(GREEN, &format!("  ·  ✓ {finished} finished")));
        }
        put(&mut buf, 2, 2, &line);
        return buf;
    };
    let others = trails
        .iter()
        .filter(|t| t.id() != trail.id() && m.seen.news(t.id()))
        .count();
    let moved = match others {
        0 => String::new(),
        1 => " 1 other trail moved".to_string(),
        n => format!(" {n} other trails moved"),
    };
    let (mark, look) = m.mark(trail);
    let mut x = put(
        &mut buf,
        2,
        0,
        &[
            paint(DIM, "All trails"),
            paint(DIM, "  ›  "),
            paint(Style::new(), "📁 "),
            paint(BOLD, &trail.record().anchor.name),
            paint(DIM, "  ›  "),
            paint(look, mark),
            paint(Style::new(), " "),
        ],
    );
    let room = usize::from(width.saturating_sub(x + moved.chars().count() as u16 + 4));
    x = put(&mut buf, x, 0, &[paint(BOLD, &cut(trail.title(), room))]);
    let _ = x;
    if others > 0 {
        let pulse = 0.5 + 0.5 * (m.t() * 6.0).sin();
        let cyan = CYAN.fg.unwrap_or_default();
        let dot = Style::new().fg(mix(toward(cyan, 0.7), cyan, pulse));
        let at = width.saturating_sub(2 + moved.chars().count() as u16 + 1);
        put(&mut buf, at, 0, &[paint(dot, "●"), paint(DIM, &moved)]);
    }
    let steps: Vec<Step> = trail.tree().into_iter().skip(1).collect();
    put(&mut buf, 2, 2, &tally(&steps, m.spin(), true));
    let done = steps.iter().filter(|s| s.state() == State::Done).count();
    let tail = format!("  {done}/{} · {}", steps.len(), worked(trail, m.at));
    let right = width.saturating_sub(2 + tail.chars().count() as u16);
    let (d, w) = m.shares(&steps);
    put(&mut buf, right.saturating_sub(24), 2, &bar(d, w, 24).spans);
    buf.set_string(right, 2, tail, Style::new());
    buf
}

/// The graph of `trail`, `width` wide: its cards chained left to right and wrapped into rows, each
/// fading in, the edge into each in the colour of the card it leads to, and a thin line back from the
/// end of a row to the start of the next; the rows of the selected card or row, else of the newest
/// card while the view follows it. It keeps in `view` where each card stood, the steps the selected
/// card lists, and the newest card.
fn chain(trail: Step, width: u16, view: &mut View, m: &Moment) -> (Buffer, Option<(u16, u16)>) {
    let tops = Trail::new(trail).parts();
    let layout = Layout::of(&tops, columns(width), &view.opened, m);
    let mut buf = Buffer::empty(Rect::new(0, 0, width, layout.height.max(1)));
    view.cards = layout
        .places
        .iter()
        .map(|p| (p.step.id().to_string(), p.x, p.y))
        .collect();
    let newest = tops
        .iter()
        .max_by_key(|s| s.newest())
        .map(|s| s.id().to_string());
    view.newest = newest.clone();
    if view
        .selected
        .as_ref()
        .is_some_and(|s| !tops.iter().any(|t| t.id() == s))
    {
        (view.selected, view.row) = (None, None);
    }
    view.listed.clear();
    let mut focus = None;
    for (i, place) in layout.places.iter().enumerate() {
        let alpha = match m.seen.added(place.step.id(), m.now) {
            Some(since) => ease(since / 0.4),
            None => ease((m.t() - 0.03 * i as f64) / 0.4),
        };
        let id = place.step.id();
        let selected = view.selected.as_deref() == Some(id);
        let opened = view.opened.contains(id);
        let card = Card {
            step: place.step,
            area: Rect::new(place.x, place.y, breadth(place.span), place.height),
            opened,
            selected,
            row: if selected { view.row.as_deref() } else { None },
            alpha,
        };
        let rows = card.draw(&mut buf, m);
        if selected && opened {
            let inner = usize::from(breadth(place.span).saturating_sub(4));
            view.listed = inside(place.step, m, inner)
                .into_iter()
                .filter(|r| r.first)
                .map(|r| r.step.id().to_string())
                .collect();
        }
        if selected {
            let row = view.row.as_ref().and_then(|r| rows.get(r));
            focus = Some(row.map_or((place.y, place.y + place.height), |y| (*y, *y + 1)));
        } else if view.selected.is_none() && view.follow && newest.as_deref() == Some(id) {
            focus = Some((place.y, place.y + place.height));
        }
        if i > 0 {
            edge(&mut buf, &layout, i, alpha);
        }
    }
    if view.row.as_ref().is_some_and(|r| !view.listed.contains(r)) {
        view.row = None;
    }
    (buf, focus)
}

/// Draws the edge into the card `i` of `layout`: an arrow from the card before it in its row, else a
/// thin, dimmer line from the end of the row before around to its left; `━` in the colour of a card
/// that works or is done, `┄` dim before one that is open or ended.
fn edge(buf: &mut Buffer, layout: &Layout, i: usize, alpha: f64) {
    let place = layout.places[i];
    let before = layout.places[i - 1];
    let state = place.step.state();
    let (line, look) = match state {
        State::Done | State::Working => ("━", lane(state)),
        _ => ("┄", DIM),
    };
    if before.y == place.y {
        let arrow = format!("{}▶", line.repeat(usize::from(GAP) - 1));
        buf.set_string(place.x - GAP, place.y + 2, arrow, look);
        if alpha < 1.0 {
            card::fade(buf, Rect::new(place.x - GAP, place.y + 2, GAP, 1), alpha);
        }
        return;
    }
    let look = crate::terminal::unfold::faded_style(look, 0.5);
    let (thin, down) = match line {
        "━" => ("─", "│"),
        _ => ("┄", "┆"),
    };
    let w = buf.area.width;
    let right = before.x + breadth(before.span);
    let turn = w.saturating_sub(4);
    let back = before.y + before.row;
    buf.set_string(
        right,
        before.y + 2,
        format!("{}╮", thin.repeat(usize::from(turn.saturating_sub(right)))),
        look,
    );
    for y in before.y + 3..back {
        buf.set_string(turn, y, down, look);
    }
    buf.set_string(
        0,
        back,
        format!("╭{}╯", thin.repeat(usize::from(turn.saturating_sub(1)))),
        look,
    );
    for y in back + 1..place.y + 2 {
        buf.set_string(0, y, down, look);
    }
    buf.set_string(0, place.y + 2, "╰▶", look);
}

/// The rules `trails` learned, newest first, naming each rule's trail when `many`.
fn learned<'a>(model: &Model<'a>, trails: &[Step<'a>], many: bool) -> Rules<'a> {
    let mut learned: Vec<Learned> = vec![];
    for trail in trails {
        for (note, rule) in model.rules.of(*trail) {
            learned.push(Learned {
                id: note.id.clone(),
                text: rule.content.clone(),
                at: note.created_at,
                trail: *trail,
            });
        }
    }
    learned.sort_by(|a, b| b.at.cmp(&a.at).then_with(|| b.id.cmp(&a.id)));
    Rules { learned, many }
}

/// The ids of the rules `model` holds for `trail`.
fn rule_ids(model: &Model, trail: Step) -> Vec<String> {
    model
        .rules
        .of(trail)
        .into_iter()
        .map(|(note, _)| note.id.clone())
        .collect()
}

/// The time worked on `trail` at the second `at`: what its done steps with no steps under them took,
/// and what those at work have spent so far.
pub(super) fn worked(trail: Step, at: u64) -> String {
    let mut total = 0;
    for leaf in trail.tree().into_iter().filter(|s| s.children().is_empty()) {
        match (Grade::of(leaf), leaf.start()) {
            (Some(grade), _) => total += grade.took,
            (None, Some(start)) if leaf.state() == State::Working => {
                total += at.saturating_sub(start.created_at);
            }
            _ => {}
        }
    }
    took(total)
}

/// Every trail of today, or the trail of the step `id`, as it stands, as plain text `PRINTED` wide: the
/// trail's graph when there is one trail or `id` names it, else All; `EMPTY` with no trail.
fn printed(model: &Model, id: Option<&str>) -> Result<String, Failure> {
    let trails = folders(model.steps);
    let named = match id {
        Some(id) => Some(model.steps.find(id)?.trail().id().to_string()),
        None => None,
    };
    if trails.is_empty() && named.is_none() {
        return Ok(format!("{EMPTY}\n"));
    }
    let mut trails = trails;
    if let Some(named) = &named {
        if !trails.iter().any(|t| t.id() == named) {
            trails.push(model.steps.find(named)?);
        }
    }
    let mut seen = Seen::settled();
    seen.note(&trails, |t| rule_ids(model, t), Instant::now());
    let mut view = View::new(&trails, named.as_deref());
    view.follow = false;
    let m = Moment {
        seen: &seen,
        now: Instant::now(),
        unfolded: &Default::default(),
        at: model.steps.now(),
    };
    let shown = view
        .trail
        .as_deref()
        .and_then(|id| trails.iter().find(|t| t.id() == id))
        .copied();
    let mut out = lines(&head(&trails, &trails, shown, &view, PRINTED, &m));
    let (canvas, ruled) = match (view.mode(), shown) {
        (Mode::Trail, Some(trail)) => (chain(trail, PRINTED, &mut view, &m).0, vec![trail]),
        _ => {
            view.pick = None;
            (
                Overview::draw(&trails, PRINTED, &view, &m).buf,
                trails.clone(),
            )
        }
    };
    out.extend(lines(&canvas));
    let rules = learned(model, &ruled, view.mode() == Mode::All);
    if let Some(pinned) = rules.pinned(PRINTED, u16::MAX, &m) {
        out.extend(lines(&pinned));
    }
    Ok(out
        .iter()
        .map(|r| format!("{}\n", r.to_string().trim_end()))
        .collect())
}

/// The page of no trail.
fn empty() -> Page {
    Page {
        lines: vec![Styled::from(EMPTY)],
        ..Page::default()
    }
}

mod card;
mod folders;
mod layout;
mod moment;
mod overview;
mod rules;
mod seen;
#[cfg(test)]
mod tests;
mod view;
