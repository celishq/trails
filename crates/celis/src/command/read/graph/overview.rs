//! All trails at once: each trail of today under its project's folder, with its front of cards.

use super::card::{closed, fade, frame, put, titled};
use super::layout::{tracks, GAP};
use super::moment::{tally, Moment, LIGHT, SEL};
use super::view::View;
use crate::command::read::tree::wrap;
use crate::terminal::bar::bar;
use crate::terminal::canvas::{paint, BOLD, CYAN, DIM, GREEN, RED};
use crate::terminal::grid::{Grid, TRACKS};
use crate::terminal::look::lane;
use crate::terminal::unfold::toward;
use crate::text::line::plural;
use celis_store::records::id::short;
use celis_trails::{State, Step, Trail};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use std::collections::HashMap;

/// All trails at once: the buffer drawn, the row each trail starts at and the rows it takes, and the
/// rows of the trail picked.
pub(super) struct Overview {
    pub buf: Buffer,
    pub places: HashMap<String, u16>,
    pub heights: HashMap<String, u16>,
    pub focus: Option<(u16, u16)>,
}

impl Overview {
    /// Draws `trails`, `width` wide, at the moment `m`: each under its project's 📁 heading with how
    /// many trails it holds, its mark, title, bar and done of all with the time worked, its steps by
    /// state, and its front as a row of small cards, the cards before folded into one and those after
    /// counted; the trail `view` picked shaded while it has the keys.
    pub fn draw(trails: &[Step], width: u16, view: &View, m: &Moment) -> Overview {
        let projects: Vec<&str> = {
            let mut seen: Vec<&str> = vec![];
            for t in trails {
                let p = t.record().anchor.project.as_str();
                if !seen.contains(&p) {
                    seen.push(p);
                }
            }
            seen
        };
        let height =
            trails.iter().map(|t| tall(*t, width)).sum::<u16>() + projects.len() as u16 * 2;
        let mut buf = Buffer::empty(Rect::new(0, 0, width, height.max(1)));
        let (mut places, mut heights, mut focus) = (HashMap::new(), HashMap::new(), None);
        let (mut y, mut project) = (0, None);
        for trail in trails {
            let anchor = &trail.record().anchor;
            if project != Some(anchor.project.as_str()) {
                project = Some(anchor.project.as_str());
                let many = trails
                    .iter()
                    .filter(|t| t.record().anchor.project == anchor.project)
                    .count();
                let plural = if many == 1 { "" } else { "s" };
                put(
                    &mut buf,
                    2,
                    y,
                    &[
                        paint(Style::new(), "📁 "),
                        paint(BOLD, &anchor.name),
                        paint(DIM, &format!("  ·  {many} trail{plural}")),
                    ],
                );
                y += 2;
            }
            places.insert(trail.id().to_string(), y);
            let rows = tall(*trail, width);
            heights.insert(trail.id().to_string(), rows);
            let picked = view.pick.as_deref() == Some(trail.id());
            if picked {
                focus = Some((y, y + rows - 1));
                if view.query.is_none() || !view.matches.is_empty() {
                    buf.set_style(
                        Rect::new(1, y, width.saturating_sub(2), heading(*trail, width)),
                        Style::new().bg(SEL),
                    );
                }
            }
            line(&mut buf, *trail, y, width, picked, m);
            strip(&mut buf, *trail, y + heading(*trail, width) + 1, width, m);
            y += rows;
        }
        Overview {
            buf,
            places,
            heights,
            focus,
        }
    }
}

/// The columns the title of `trail` wraps in on a line `width` wide, before its bar and done of all with
/// room for the longest time worked, so the title keeps its rows while the time counts.
fn room(trail: Step, width: u16) -> usize {
    let steps = trail.tree().len().saturating_sub(1);
    let done = trail
        .tree()
        .into_iter()
        .skip(1)
        .filter(|s| s.state() == State::Done)
        .count();
    let tail = format!("  {done}/{steps} · 00h 00m 00s").chars().count() as u16;
    usize::from(width.saturating_sub(2 + tail + 6 + 22))
}

/// The rows the title of `trail` takes wrapped on a line `width` wide, one at least.
fn heading(trail: Step, width: u16) -> u16 {
    wrap(trail.title(), room(trail, width)).len().max(1) as u16
}

/// Draws the lines of `trail` from row `y`: a dot while it just moved, its mark, its whole title
/// wrapped, and at the right of its first row its bar or `finished`, and done of all with the time
/// worked; under the title its steps by state.
fn line(buf: &mut Buffer, trail: Step, y: u16, width: u16, picked: bool, m: &Moment) {
    if let Some(since) = m.seen.moved(trail.id(), m.now).filter(|s| *s < 1.2) {
        let dot = Style::new().fg(toward(CYAN.fg.unwrap_or_default(), 1.0 - since / 1.2));
        buf.set_string(2, y, "●", dot);
    }
    let (mark, look) = m.mark(trail);
    buf.set_string(4, y, mark, look);
    let steps: Vec<Step> = trail.tree().into_iter().skip(1).collect();
    let done = steps.iter().filter(|s| s.state() == State::Done).count();
    let tail = format!("  {done}/{} · {}", steps.len(), super::worked(trail, m.at));
    let right = width.saturating_sub(2 + tail.chars().count() as u16);
    let state = trail.state();
    let ended = state.closed() || state == State::Expired;
    let title = match (picked, ended) {
        (true, _) => Style::new().fg(LIGHT).patch(BOLD),
        (false, true) => DIM.patch(BOLD),
        (false, false) => BOLD,
    };
    for (n, text) in wrap(trail.title(), room(trail, width))
        .into_iter()
        .enumerate()
    {
        buf.set_string(6, y + n as u16, text, title);
    }
    let end = match state {
        State::Done => Some(("finished", GREEN)),
        State::Dropped => Some(("dropped", RED)),
        State::Expired => Some(("expired", DIM)),
        _ => None,
    };
    match end {
        Some((word, look)) => {
            buf.set_string(right.saturating_sub(word.len() as u16), y, word, look);
        }
        None => {
            let (d, w) = m.shares(&steps);
            put(buf, right.saturating_sub(20), y, &bar(d, w, 20).spans);
        }
    }
    buf.set_string(right, y, tail, Style::new());
    put(
        buf,
        6,
        y + heading(trail, width),
        &tally(&steps, m.spin(), true),
    );
}

/// The front of `trail` on a row `width` wide: from the last closed card before the first still at
/// work, as many as fit with the cards before folded into one; the cards folded before, the cards shown
/// each with its column and width, and how many come after.
fn front<'a>(trail: Step<'a>, width: u16) -> (Vec<Step<'a>>, Vec<(Step<'a>, u16, u16)>, usize) {
    let tops = Trail::new(trail).parts();
    let grid = Grid::of(width);
    let per = tracks(grid);
    let fit = TRACKS / per;
    let first = tops
        .iter()
        .position(|s| matches!(s.state(), State::Working | State::Open))
        .unwrap_or(tops.len().saturating_sub(1));
    let mut start = first.saturating_sub(1);
    let room = |start: usize| usize::from(fit) - usize::from(start > 0);
    if start > 0 && start + room(start) > tops.len() {
        start = tops.len().saturating_sub(room(start));
    }
    let cards = &tops[start..(start + room(start)).min(tops.len())];
    let after = tops.len() - start - cards.len();
    let place = |n: usize| grid.span(n as u16 * per, (n as u16 + 1) * per);
    let shown = cards
        .iter()
        .enumerate()
        .map(|(n, step)| {
            let (x, w) = place(n + usize::from(start > 0));
            (*step, x, w)
        })
        .collect();
    (tops[..start].to_vec(), shown, after)
}

/// The rows the front of `trail` takes `width` wide: as tall as its tallest small card, its whole
/// title, a row under it and its border.
fn strip_rows(trail: Step, width: u16) -> u16 {
    let (_, cards, _) = front(trail, width);
    let title = cards
        .iter()
        .map(|(step, _, w)| titled(*step, usize::from(w.saturating_sub(4))))
        .max()
        .unwrap_or(1);
    title + 3
}

/// The rows `trail` takes on All: its title, its steps by state, its front, and a gap.
fn tall(trail: Step, width: u16) -> u16 {
    heading(trail, width) + 1 + strip_rows(trail, width) + 2
}

/// Draws at row `y` the front of `trail` as small cards as `front` gives them, each as tall as the
/// tallest, and how many come after.
fn strip(buf: &mut Buffer, trail: Step, y: u16, width: u16, m: &Moment) {
    let (before, cards, after) = front(trail, width);
    if cards.is_empty() {
        buf.set_string(Grid::of(width).column(0), y + 1, "no steps", DIM);
        return;
    }
    let height = strip_rows(trail, width);
    if !before.is_empty() {
        let grid = Grid::of(width);
        let (x, w) = grid.span(0, tracks(grid));
        let under: Vec<Step> = before.iter().flat_map(|s| s.tree()).collect();
        let area = Rect::new(x, y, w, height);
        frame(buf, area, DIM, true);
        put(
            buf,
            x + 2,
            y,
            &[
                paint(Style::new(), " ▸ "),
                paint(GREEN, "✓"),
                paint(DIM, &format!(" {} before ", before.len())),
            ],
        );
        put(buf, x + 2, y + 1, &tally(&before, m.spin(), false));
        buf.set_string(x + 2, y + 2, plural(under.len(), "step"), DIM);
    }
    let mut end = 0;
    for (n, (step, x, w)) in cards.iter().enumerate() {
        let area = Rect::new(*x, y, *w, height);
        small(buf, *step, area, m);
        if n > 0 || !before.is_empty() {
            let state = step.state();
            let (glyph, look) = match state {
                State::Done | State::Working => ("━", lane(state)),
                _ => ("┄", DIM),
            };
            let edge = format!("{}▶", glyph.repeat(usize::from(GAP) - 1));
            buf.set_string(x - GAP, y + 2, edge, look);
        }
        end = x + w;
    }
    if after > 0 {
        buf.set_string(end, y + 2, "┄", DIM);
        let more = format!("+{after} after");
        buf.set_string(
            width.saturating_sub(2 + more.len() as u16),
            y - 1,
            more,
            DIM,
        );
    }
}

/// Draws the small card of `step` over `area`: its mark and id in its border, its title, and under it
/// its counts and time when closed, its bar while it works with steps, else its state.
fn small(buf: &mut Buffer, step: Step, area: Rect, m: &Moment) {
    let state = step.state();
    frame(buf, area, m.border(step, false), state == State::Open);
    let (mark, look) = m.mark(step);
    let mut label = vec![paint(Style::new(), " ")];
    if !step.children().is_empty() {
        label.push(paint(Style::new(), "▸ "));
    }
    label.push(paint(look, mark));
    label.extend(m.sign(step));
    label.push(paint(DIM, &format!(" {} ", short(step.id()))));
    put(buf, area.x + 2, area.y, &label);
    let inner = usize::from(area.width - 4);
    let x = area.x + 2;
    match state {
        State::Done | State::Dropped | State::Expired => closed(buf, step, area, m),
        _ => {
            let look = if state == State::Working {
                BOLD
            } else {
                m.tone(state)
            };
            for (n, line) in wrap(step.title(), inner).into_iter().enumerate() {
                buf.set_string(x, area.y + 1 + n as u16, line, look);
            }
            let y = area.y + 1 + titled(step, inner);
            let under: Vec<Step> = step.tree().into_iter().skip(1).collect();
            if state == State::Working && !under.is_empty() {
                let (d, w) = m.shares(&under);
                let end = put(buf, x, y, &bar(d, w, 10).spans);
                let done = under.iter().filter(|s| s.state() == State::Done).count();
                buf.set_string(end + 1, y, format!("{done}/{}", under.len()), DIM);
            } else {
                buf.set_string(x, y, state.text(), DIM);
            }
        }
    }
    let alpha = m.arrived(m.seen.added(step.id(), m.now), 0.4);
    if alpha < 1.0 {
        fade(buf, area, alpha);
    }
}
