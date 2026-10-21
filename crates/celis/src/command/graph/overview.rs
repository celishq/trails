//! All trails at once: each trail of today under its project's folder, with its front of cards.

use super::card::{closed, fade, frame, put, WIDTH};
use super::layout::{columns, GAP, LEFT};
use super::moment::{tally, Moment, LIGHT, SEL};
use super::view::View;
use crate::terminal::bar::bar;
use crate::terminal::canvas::{cut, paint, BOLD, CYAN, DIM, GREEN, RED};
use crate::terminal::look::lane;
use crate::terminal::unfold::toward;
use crate::text::line::plural;
use celis_trails::records::id::short;
use celis_trails::{State, Step, Trail};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use std::collections::HashMap;

/// The rows a trail takes on All: its line, its steps by state, its cards, and a gap.
pub(super) const TALL: u16 = 8;

/// All trails at once: the buffer drawn, the row each trail starts at, and the rows of the trail
/// picked.
pub(super) struct Overview {
    pub buf: Buffer,
    pub places: HashMap<String, u16>,
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
        let height = trails.len() as u16 * TALL + projects.len() as u16 * 2;
        let mut buf = Buffer::empty(Rect::new(0, 0, width, height.max(1)));
        let (mut places, mut focus) = (HashMap::new(), None);
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
            let picked = view.pick.as_deref() == Some(trail.id());
            if picked {
                focus = Some((y, y + TALL - 1));
                if view.query.is_none() || !view.matches.is_empty() {
                    buf.set_style(
                        Rect::new(1, y, width.saturating_sub(2), 1),
                        Style::new().bg(SEL),
                    );
                }
            }
            line(&mut buf, *trail, y, width, picked, m);
            strip(&mut buf, *trail, y + 2, width, m);
            y += TALL;
        }
        Overview { buf, places, focus }
    }
}

/// Draws the line of `trail` at row `y`: a dot while it just moved, its mark, title, and at the right
/// its bar or `finished`, and done of all with the time worked; under it its steps by state.
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
    let room = usize::from(right.saturating_sub(6 + 22));
    buf.set_string(6, y, cut(trail.title(), room), title);
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
    put(buf, 6, y + 1, &tally(&steps, m.spin(), true));
}

/// Draws at row `y` the front of `trail` as small cards: from the last closed card before the first
/// still at work, as many as fit with the cards before folded into one, and how many come after.
fn strip(buf: &mut Buffer, trail: Step, y: u16, width: u16, m: &Moment) {
    let tops = Trail::new(trail).parts();
    let fit = columns(width.saturating_sub(2));
    let first = tops
        .iter()
        .position(|s| matches!(s.state(), State::Working | State::Open))
        .unwrap_or(tops.len().saturating_sub(1));
    let mut start = first.saturating_sub(1);
    let room = |start: usize| usize::from(fit) - usize::from(start > 0);
    if start > 0 && start + room(start) > tops.len() {
        start = tops.len().saturating_sub(room(start));
    }
    if tops.is_empty() {
        buf.set_string(LEFT + 4, y + 1, "no steps", DIM);
        return;
    }
    let cards = &tops[start..(start + room(start)).min(tops.len())];
    let after = tops.len() - start - cards.len();
    let w = WIDTH - 1;
    let mut x = LEFT + 2;
    if start > 0 {
        let before = &tops[..start];
        let under: Vec<Step> = before.iter().flat_map(|s| s.tree()).collect();
        let area = Rect::new(x, y, w, 4);
        frame(buf, area, DIM, true);
        put(
            buf,
            x + 2,
            y,
            &[
                paint(Style::new(), " ▸ "),
                paint(GREEN, "✓"),
                paint(DIM, &format!(" {start} before ")),
            ],
        );
        put(buf, x + 2, y + 1, &tally(before, m.spin(), false));
        buf.set_string(x + 2, y + 2, plural(under.len(), "step"), DIM);
        x += w + GAP;
    }
    for (n, step) in cards.iter().enumerate() {
        let area = Rect::new(x, y, w, 4);
        small(buf, *step, area, m);
        if n > 0 || start > 0 {
            let state = step.state();
            let (glyph, look) = match state {
                State::Done | State::Working => ("━", lane(state)),
                _ => ("┄", DIM),
            };
            let edge = format!("{}▶", glyph.repeat(usize::from(GAP) - 1));
            buf.set_string(x - GAP, y + 2, edge, look);
        }
        x += w + GAP;
    }
    if after > 0 {
        buf.set_string(x - GAP + 1, y + 2, "┄", DIM);
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
            buf.set_string(x, area.y + 1, cut(step.title(), inner), look);
            let under: Vec<Step> = step.tree().into_iter().skip(1).collect();
            if state == State::Working && !under.is_empty() {
                let (d, w) = m.shares(&under);
                let end = put(buf, x, area.y + 2, &bar(d, w, 10).spans);
                let done = under.iter().filter(|s| s.state() == State::Done).count();
                buf.set_string(end + 1, area.y + 2, format!("{done}/{}", under.len()), DIM);
            } else {
                buf.set_string(x, area.y + 2, state.text(), DIM);
            }
        }
    }
    let alpha = m.arrived(m.seen.added(step.id(), m.now), 0.4);
    if alpha < 1.0 {
        fade(buf, area, alpha);
    }
}
