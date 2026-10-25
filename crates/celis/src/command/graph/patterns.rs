//! The patterns card of the graph view: the patterns the trails shown learned, newest first.

use super::card::{fade, frame, put};
use super::moment::{Moment, LIGHT, SEL};
use super::view::View;
use crate::command::tree::wrap;
use crate::terminal::canvas::{paint, BOLD, DIM, EDGE, MAGENTA};
use crate::text::clock::clock;
use celis_trails::records::id::short;
use celis_trails::Step;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

/// One pattern a trail learned: its id, its text, the second it was written, and its trail.
pub(super) struct Learned<'a> {
    pub id: String,
    pub text: String,
    pub at: u64,
    pub trail: Step<'a>,
}

/// The patterns card: the patterns learned, newest first, and whether it names each pattern's trail.
pub(super) struct Patterns<'a> {
    pub learned: Vec<Learned<'a>>,
    pub many: bool,
}

impl Patterns<'_> {
    /// The card pinned under the graph, `width` wide and at most `room` rows: as many patterns as fit
    /// whole, and how many older ones it leaves out; none with no pattern.
    pub fn pinned(&self, width: u16, room: u16, m: &Moment) -> Option<Buffer> {
        if self.learned.is_empty() {
            return None;
        }
        let text = usize::from(width.saturating_sub(22));
        let mut lines: Vec<(usize, usize, String)> = vec![];
        for (n, pattern) in self.learned.iter().enumerate() {
            for (k, line) in wrap(&pattern.text, text).into_iter().enumerate() {
                lines.push((n, k, line));
            }
        }
        let mut shown = lines.len().min(usize::from(room.saturating_sub(2)).max(1));
        while shown < lines.len() && lines[shown].1 != 0 {
            shown -= 1;
        }
        let kept = lines[..shown].last().map_or(0, |l| l.0 + 1);
        let rest = self.learned.len() - kept;
        let mut title = vec![
            paint(BOLD.fg(MAGENTA.fg.unwrap_or_default()), " § Patterns "),
            paint(DIM, &format!("{} ", self.learned.len())),
        ];
        if rest > 0 {
            title.push(paint(DIM, &format!("· {rest} older not shown ")));
        }
        let mut buf = Buffer::empty(Rect::new(0, 0, width, shown as u16 + 2));
        frame(
            &mut buf,
            Rect::new(2, 0, width.saturating_sub(4), shown as u16 + 2),
            EDGE,
            false,
        );
        put(&mut buf, 4, 0, &title);
        for (row, (n, k, line)) in lines[..shown].iter().enumerate() {
            let y = row as u16 + 1;
            self.line(&mut buf, *n, *k, line, y, m, false);
        }
        Some(buf)
    }

    /// The card opened over the screen, `width` wide and `room` rows: every pattern with its trail when it
    /// names them and when it was learned, the pattern `view` selected shaded, scrolled to show it; it
    /// keeps the first row shown in `view`.
    pub fn opened(&self, width: u16, room: u16, view: &mut View, m: &Moment) -> Buffer {
        let inner = room.saturating_sub(2).max(1);
        let mut buf = Buffer::empty(Rect::new(0, 0, width, room.max(3)));
        let mut title = vec![
            paint(BOLD.fg(MAGENTA.fg.unwrap_or_default()), " § Patterns "),
            paint(DIM, &format!("{} ", self.learned.len())),
        ];
        if self.learned.is_empty() {
            frame(
                &mut buf,
                Rect::new(2, 0, width.saturating_sub(4), 3),
                EDGE,
                false,
            );
            put(&mut buf, 4, 0, &title);
            let none = match self.many {
                true => "No pattern learned in these trails yet.",
                false => "No pattern learned in this trail yet.",
            };
            buf.set_string(4, 1, none, DIM);
            return buf;
        }
        let text = usize::from(width.saturating_sub(22));
        let mut lines: Vec<(usize, Option<(usize, String)>)> = vec![];
        for (n, pattern) in self.learned.iter().enumerate() {
            lines.extend(
                wrap(&pattern.text, text)
                    .into_iter()
                    .enumerate()
                    .map(|l| (n, Some(l))),
            );
            lines.push((n, None));
        }
        let picked = self
            .learned
            .iter()
            .position(|r| Some(&r.id) == view.pattern.as_ref())
            .unwrap_or(0);
        let at = lines.iter().position(|l| l.0 == picked).unwrap_or(0);
        let size = lines.iter().filter(|l| l.0 == picked).count();
        let mut top = view.pattern_top.min(at);
        if at + size > top + usize::from(inner) {
            top = at + size - usize::from(inner);
        }
        view.pattern_top = top;
        let shown: Vec<&(usize, Option<(usize, String)>)> =
            lines.iter().skip(top).take(usize::from(inner)).collect();
        title.push(paint(
            DIM,
            &format!("· {} of {} ", picked + 1, self.learned.len()),
        ));
        let height = shown.len() as u16 + 2;
        frame(
            &mut buf,
            Rect::new(2, 0, width.saturating_sub(4), height),
            EDGE,
            false,
        );
        put(&mut buf, 4, 0, &title);
        let below = lines.len().saturating_sub(top + shown.len());
        if top > 0 {
            buf.set_string(width.saturating_sub(16), 0, format!(" ▲ {top} rows "), DIM);
        }
        if below > 0 {
            buf.set_string(
                width.saturating_sub(16),
                height - 1,
                format!(" ▼ {below} rows "),
                DIM,
            );
        }
        for (row, (n, line)) in shown.iter().enumerate() {
            let y = row as u16 + 1;
            if *n == picked {
                buf.set_style(
                    Rect::new(3, y, width.saturating_sub(6), 1),
                    Style::new().bg(SEL),
                );
            }
            match line {
                Some((k, line)) => self.line(&mut buf, *n, *k, line, y, m, *n == picked),
                None => {
                    let pattern = &self.learned[*n];
                    let mut meta = match self.many {
                        true => format!(
                            "{} · ",
                            crate::terminal::canvas::cut(pattern.trail.title(), 30)
                        ),
                        false => String::new(),
                    };
                    meta += &format!("learned {}", clock(pattern.at, m.at));
                    let x = width.saturating_sub(4 + meta.chars().count() as u16);
                    buf.set_string(x, y, meta, DIM);
                }
            }
        }
        buf
    }

    /// Draws the line `line`, the `k`th of the pattern `n`, at row `y`: on its first, the `§` glowing as
    /// it lands and the pattern's id, then its text in the light when `lit`; fading in as it lands.
    #[allow(clippy::too_many_arguments)]
    fn line(
        &self,
        buf: &mut Buffer,
        n: usize,
        k: usize,
        line: &str,
        y: u16,
        m: &Moment,
        lit: bool,
    ) {
        let pattern = &self.learned[n];
        let landed = m.seen.landed(&pattern.id, m.now);
        if k == 0 {
            buf.set_string(4, y, "§", m.pattern(landed));
            buf.set_string(6, y, short(&pattern.id), DIM);
        }
        let look = match lit {
            true => Style::new().fg(LIGHT),
            false => Style::new(),
        };
        buf.set_string(16, y, line, look);
        let alpha = m.arrived(landed, 0.4);
        if alpha < 1.0 {
            fade(
                buf,
                Rect::new(4, y, buf.area.width.saturating_sub(8), 1),
                alpha,
            );
        }
    }
}
