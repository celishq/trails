//! The card of one step of a trail's graph: closed to its counts, at work with its own steps, or opened
//! by the person to every step under it.

use super::moment::{tally, Moment, LIGHT, SEL};
use crate::command::read::tree::wrap;
use crate::progress::{Grade, Phase};
use crate::terminal::bar::bar;
use crate::terminal::canvas::{cut, paint, BOLD, DIM};
use crate::terminal::look::lane;
use crate::terminal::unfold::faded_style;
use crate::text::clock::took;
use celis_store::records::id::short;
use celis_trails::{State, Step};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;

/// The width of a card in the chain.
pub(super) const WIDTH: u16 = 23;

/// The most steps a working card lists.
const ROWS: usize = 4;

/// The card of `step` over `area`: whether the person opened it and selected it or a row in it, the row
/// selected, and how far it has faded in.
pub(super) struct Card<'a> {
    pub step: Step<'a>,
    pub area: Rect,
    pub opened: bool,
    pub selected: bool,
    pub row: Option<&'a str>,
    pub alpha: f64,
}

/// A row of an opened card: its cells, the meta at its right, the step it shows, and whether it is that
/// step's first row.
pub(super) struct Row<'a> {
    pub cells: Vec<Span<'static>>,
    pub meta: String,
    pub step: Step<'a>,
    pub first: bool,
}

impl<'a> Card<'a> {
    /// The rows a card of `step` takes, `width` wide: a closed card's two, an open card's title and
    /// state or the bar of its steps done, a working card's title, state, rows and bar; a card the person opened, its whole title and
    /// the rows of the steps under it it unfolded.
    pub fn height(step: Step<'a>, opened: bool, width: u16, m: &Moment) -> u16 {
        let inner = usize::from(width.saturating_sub(4));
        let shown = !step.children().is_empty();
        if opened {
            let mut rows = 2 + wrap(step.title(), inner).len() + 1;
            if shown {
                rows += 1 + inside(step, m, inner).len() + 1;
            }
            return rows as u16;
        }
        match Phase::of(&step) {
            Phase::Done | Phase::Dropped | Phase::Stale => 4,
            Phase::Started if shown => {
                let (rows, more) = listed(step);
                (5 + 2 + rows.len() + usize::from(more > 0)) as u16
            }
            _ => 5,
        }
    }

    /// Draws it on `buf` at the moment `m`; the row of each step it lists, by its id.
    pub fn draw(&self, buf: &mut Buffer, m: &Moment) -> HashMap<String, u16> {
        let (step, area) = (self.step, self.area);
        let state = Phase::of(&step);
        let border = m.border(step, self.selected && self.row.is_none());
        let dashed = matches!(state, Phase::Open) && !self.selected;
        frame(buf, area, border, dashed);
        let mut label = vec![paint(Style::new(), " ")];
        if !step.children().is_empty() {
            let toggle = if self.opened { "▾ " } else { "▸ " };
            label.push(paint(lit(self.selected, Style::new()), toggle));
        }
        let (mark, look) = m.mark(step);
        label.push(paint(look, mark));
        label.extend(m.sign(step));
        let id = format!(" {} ", short(step.id()));
        label.push(paint(lit(self.selected, DIM), &id));
        put(buf, area.x + 2, area.y, &label);
        let inner = usize::from(area.width.saturating_sub(4));
        let x = area.x + 2;
        let mut rows = HashMap::new();
        if self.opened {
            self.opened_body(buf, m, inner, &mut rows);
        } else if matches!(state, Phase::Done | Phase::Dropped | Phase::Stale) {
            closed(buf, step, area, m);
        } else {
            let titles = wrap(step.title(), inner);
            let look = match state {
                Phase::Started => BOLD,
                _ => m.tone(state),
            };
            for (n, line) in titles.iter().take(2).enumerate() {
                let line = match (n, titles.len() > 2) {
                    (1, true) => cut(&format!("{line} …"), inner),
                    _ => line.clone(),
                };
                buf.set_string(x, area.y + 1 + n as u16, line, look);
            }
            match state {
                Phase::Started => {
                    let meta = format!("open · {}", spent(step, m.at));
                    buf.set_string(x, area.y + 3, meta, DIM);
                }
                _ if !step.children().is_empty() => {
                    progress(buf, x, area.y + 3, step, m, 12, false)
                }
                _ => {
                    buf.set_string(x, area.y + 3, "open", DIM);
                }
            }
            if state == Phase::Started && !step.children().is_empty() {
                self.working_body(buf, m, inner, &mut rows);
            }
        }
        if self.alpha < 1.0 {
            fade(buf, area, self.alpha);
        }
        rows
    }

    /// Draws the steps a working card lists under a line: the working first, then the open, then the
    /// newest closed, each fading in as it comes; how many it leaves out by state; and a bar of the
    /// share of its steps done.
    fn working_body(
        &self,
        buf: &mut Buffer,
        m: &Moment,
        inner: usize,
        rows: &mut HashMap<String, u16>,
    ) {
        let area = self.area;
        let x = area.x + 2;
        let line = format!("├{}┤", "╌".repeat(usize::from(area.width) - 2));
        buf.set_string(
            area.x,
            area.y + 4,
            line,
            faded_style(m.border(self.step, false), 0.6),
        );
        let (listed, more) = listed(self.step);
        let mut y = area.y + 5;
        for kid in &listed {
            let (mark, look) = m.mark(*kid);
            buf.set_string(x, y, mark, look);
            buf.set_string(
                x + 2,
                y,
                cut(kid.title(), inner - 2),
                m.tone(Phase::of(kid)),
            );
            let alpha = m.arrived(m.seen.added(kid.id(), m.now), 0.4);
            if alpha < 1.0 {
                fade(buf, Rect::new(x, y, area.width - 4, 1), alpha);
            }
            rows.insert(kid.id().to_string(), y);
            y += 1;
        }
        if more > 0 {
            let hidden: Vec<Step> = self
                .step
                .children()
                .into_iter()
                .filter(|k| !listed.iter().any(|l| l.id() == k.id()))
                .collect();
            let mut spans = vec![paint(DIM, &format!("+{more} more  "))];
            spans.extend(tally(&hidden, m.spin(), false));
            put(buf, x, y, &spans);
            y += 1;
        }
        progress(buf, x, y, self.step, m, 12, false);
    }

    /// Draws an opened card's whole title, its state and time, then under a line each step under it
    /// as a tree row, the row selected shaded, and a bar of the share done with its steps by state.
    fn opened_body(
        &self,
        buf: &mut Buffer,
        m: &Moment,
        inner: usize,
        rows: &mut HashMap<String, u16>,
    ) {
        let (step, area) = (self.step, self.area);
        let x = area.x + 2;
        let mut y = area.y + 1;
        for line in wrap(step.title(), inner) {
            buf.set_string(x, y, line, m.tone(Phase::of(&step)).patch(BOLD));
            y += 1;
        }
        let state = Phase::of(&step);
        let meta = match state {
            Phase::Done if step.changed().is_some() => {
                format!("done otherwise than planned · {}", spent(step, m.at))
            }
            Phase::Done | Phase::Started => format!("{} · {}", state.text(), spent(step, m.at)),
            _ => state.text().to_string(),
        };
        buf.set_string(x, y, meta, DIM);
        y += 1;
        if step.children().is_empty() {
            return;
        }
        let line = format!("├{}┤", "╌".repeat(usize::from(area.width) - 2));
        buf.set_string(area.x, y, line, faded_style(m.border(step, false), 0.6));
        y += 1;
        for row in inside(step, m, inner) {
            let picked = self.selected && self.row == Some(row.step.id());
            if picked {
                buf.set_style(
                    Rect::new(area.x + 1, y, area.width - 2, 1),
                    Style::new().bg(SEL),
                );
            }
            if row.first {
                rows.insert(row.step.id().to_string(), y);
            }
            let cells: Vec<Span> = row
                .cells
                .into_iter()
                .map(|s| match picked && s.style.fg.is_none() {
                    true => paint(Style::new().fg(LIGHT).patch(BOLD), &s.content),
                    false => s,
                })
                .collect();
            put(buf, x, y, &cells);
            let meta = row.meta.as_str();
            let at = x + inner as u16 - meta.width() as u16;
            buf.set_string(at, y, meta, DIM);
            y += 1;
        }
        progress(buf, x, y, step, m, 20, true);
    }
}

/// The steps right under `step` a working card lists, the working first, then the open, then the
/// newest closed, at most `ROWS`; and how many it leaves out.
fn listed(step: Step) -> (Vec<Step>, usize) {
    let mut kids = step.children();
    let rank = |s: &Step| match Phase::of(s) {
        Phase::Started => 0,
        Phase::Open => 1,
        _ => 2,
    };
    kids.sort_by_key(|s| (rank(s), std::cmp::Reverse(s.newest())));
    match kids.len() > ROWS {
        true => {
            let more = kids.len() - (ROWS - 1);
            kids.truncate(ROWS - 1);
            (kids, more)
        }
        false => (kids, 0),
    }
}

/// The steps under `step` as tree rows, `width` wide, each nesting folded under `▸` until the person
/// unfolds it: lanes in the colour of the step they hang from, each level hung from its parent's mark,
/// a space between joint, toggle, mark and title, and its id and time or state at its right.
pub(super) fn inside<'a>(step: Step<'a>, m: &Moment, width: usize) -> Vec<Row<'a>> {
    let mut rows = vec![];
    walk(step, &[], m, width, &mut rows);
    rows
}

/// Adds the rows of the steps right under `step` to `rows`, after the lanes `lead`, and of the steps
/// under each one the person unfolded.
fn walk<'a>(
    step: Step<'a>,
    lead: &[Span<'static>],
    m: &Moment,
    width: usize,
    rows: &mut Vec<Row<'a>>,
) {
    let kids = step.children();
    let colour = lane(Phase::of(&step));
    for (n, kid) in kids.iter().enumerate() {
        let last = n + 1 == kids.len();
        let nested = !kid.children().is_empty();
        let mut head = lead.to_vec();
        head.push(paint(colour, if last { "└─" } else { "├─" }));
        head.push(match (nested, m.unfolded.contains(kid.id())) {
            (true, true) => paint(Style::new(), " ▾"),
            (true, false) => paint(Style::new(), " ▸"),
            (false, _) => paint(colour, "──"),
        });
        head.push(paint(Style::new(), " "));
        let (mark, look) = m.mark(*kid);
        head.push(paint(look, mark));
        head.extend(m.sign(*kid));
        head.push(paint(Style::new(), " "));
        let used: usize = head.iter().map(|s| s.content.width()).sum();
        let state = Phase::of(kid);
        let under: Vec<Step> = kid.tree().into_iter().skip(1).collect();
        let mut meta = short(kid.id());
        meta += &match state {
            Phase::Done => format!(" · {}", spent(*kid, m.at)),
            _ => format!(" · {}", state.text()),
        };
        if !under.is_empty() {
            let done = under.iter().filter(|s| s.state() == State::Done).count();
            meta = format!("{done}/{} · {meta}", under.len());
        }
        let room = width.saturating_sub(used + meta.width() + 2).max(8);
        let lines = wrap(kid.title(), room);
        let tone = m.tone(state);
        for (k, line) in lines.iter().enumerate() {
            let mut cells = match k {
                0 => head.clone(),
                _ => {
                    let mut down = lead.to_vec();
                    down.push(paint(colour, if last { " " } else { "│" }));
                    let pad = used - lead.iter().map(|s| s.content.width()).sum::<usize>() - 1;
                    down.push(paint(Style::new(), &" ".repeat(pad)));
                    down
                }
            };
            cells.push(paint(tone, line));
            let meta = if k == 0 { meta.clone() } else { String::new() };
            rows.push(Row {
                cells,
                meta,
                step: *kid,
                first: k == 0,
            });
        }
        if nested && m.unfolded.contains(kid.id()) {
            let mut deeper = lead.to_vec();
            deeper.push(paint(colour, if last { "     " } else { "│    " }));
            walk(*kid, &deeper, m, width, rows);
        }
    }
}

/// Draws a closed card's title, and under it its steps by state and its time or why it ended.
pub(super) fn closed(buf: &mut Buffer, step: Step, area: Rect, m: &Moment) {
    let inner = usize::from(area.width.saturating_sub(4));
    let x = area.x + 2;
    buf.set_string(
        x,
        area.y + 1,
        cut(step.title(), inner),
        m.tone(Phase::of(&step)),
    );
    let under: Vec<Step> = step.tree().into_iter().skip(1).collect();
    let end = match under.is_empty() {
        true => x,
        false => put(buf, x, area.y + 2, &tally(&under, m.spin(), false)),
    };
    let state = Phase::of(&step);
    let mut right = match state {
        Phase::Done if step.changed().is_some() => format!("changed · {}", spent(step, m.at)),
        Phase::Done => spent(step, m.at),
        _ => state.text().to_string(),
    };
    if usize::from(end) + right.width() + 1 > usize::from(x) + inner {
        right = match state {
            Phase::Done => spent(step, m.at),
            _ => state.text().to_string(),
        };
    }
    let at = x + inner as u16 - right.width() as u16;
    buf.set_string(at, area.y + 2, right, DIM);
}

/// Draws at `x`, `y` a bar `cells` wide of the share of the steps under `step` done, eased as it
/// changes, then how many are done of all, and their counts by state when `counts`.
fn progress(buf: &mut Buffer, x: u16, y: u16, step: Step, m: &Moment, cells: usize, counts: bool) {
    let under: Vec<Step> = step.tree().into_iter().skip(1).collect();
    let (done, working) = m.shares(&under);
    let end = put(buf, x, y, &bar(done, working, cells).spans);
    let finished = under.iter().filter(|s| s.state() == State::Done).count();
    let end = put(
        buf,
        end + 1,
        y,
        &[paint(DIM, &format!("{finished}/{}", under.len()))],
    );
    if counts {
        put(buf, end + 2, y, &tally(&under, m.spin(), false));
    }
}

/// The time `step` took when done, or has spent so far while it works, at the second `at`.
pub(super) fn spent(step: Step, at: u64) -> String {
    match (Grade::of(step), step.start()) {
        (Some(grade), _) => took(grade.took),
        (None, Some(start)) if step.works() => took(at.saturating_sub(start.created_at)),
        _ => Phase::of(&step).text().to_string(),
    }
}

/// Draws a box over `area` of `buf` with round corners in `look`, its line dashed when `dashed`.
pub(super) fn frame(buf: &mut Buffer, area: Rect, look: Style, dashed: bool) {
    let (h, v) = if dashed {
        ("╌", "╎")
    } else {
        ("─", "│")
    };
    let inner = usize::from(area.width.saturating_sub(2));
    buf.set_string(area.x, area.y, format!("╭{}╮", h.repeat(inner)), look);
    for y in area.y + 1..area.bottom().saturating_sub(1) {
        buf.set_string(area.x, y, v, look);
        buf.set_string(area.right() - 1, y, v, look);
    }
    let bottom = area.bottom().saturating_sub(1);
    buf.set_string(area.x, bottom, format!("╰{}╯", h.repeat(inner)), look);
}

/// Writes `spans` on `buf` from `x`, `y`, each in its look; the column after the last.
pub(super) fn put(buf: &mut Buffer, x: u16, y: u16, spans: &[Span]) -> u16 {
    let mut x = x;
    for span in spans {
        let (end, _) = buf.set_stringn(x, y, &span.content, usize::from(u16::MAX), span.style);
        x = end;
    }
    x
}

/// Fades the cells of `area` on `buf` `alpha` of the way from the background to their own looks.
pub(super) fn fade(buf: &mut Buffer, area: Rect, alpha: f64) {
    let area = area.intersection(buf.area);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &mut buf[(x, y)];
            let style = faded_style(cell.style(), alpha);
            cell.fg = style.fg.unwrap_or(cell.fg);
            cell.bg = style.bg.unwrap_or(cell.bg);
            cell.modifier = style.add_modifier;
        }
    }
}

/// `look` in the text's light when `selected`.
fn lit(selected: bool, look: Style) -> Style {
    match selected {
        true => Style::new().fg(LIGHT).patch(BOLD),
        false => look,
    }
}

#[cfg(test)]
mod tests;
