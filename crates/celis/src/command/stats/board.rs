//! `celis stats` on a terminal: its cards, an opened card, or the route of a step, drawn again each
//! time a record lands in the store or a linked folder.

use super::sheet::{sheets, Sheet, CARDS};
use super::table::{Row, Table};
use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::progress::Model;
use crate::terminal::canvas::{cut, visible};
use crate::terminal::canvas::{paint, Styled, BOLD, DIM};
use crate::terminal::card::{grid, Card};
use crate::terminal::ink::Ink;
use crate::terminal::live::{Live, Page};
use crate::terminal::look::look;
use crate::terminal::screen::Screen;
use crate::terminal::wake::Key;
use crate::text::clock::weekday;
use crate::text::line::time;
use celis_nostr::Note;
use celis_trails::records::id::short;
use celis_trails::{Step, Steps, Trail};
use ratatui::text::Span;
use std::io::Write;
use std::time::{Duration, Instant};

impl Command {
    /// Draws the cards of stats on the alternate screen, or the route of the step `args` names, then
    /// again when a record lands, a key is read, a minute passes or the terminal's size changes. With
    /// no store, it writes nothing and waits for one. On an interrupt or `q`, leaves the alternate
    /// screen with nothing printed, and ends.
    pub(crate) fn board(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let view = match args.words.get(1) {
            Some(id) if !self.held()? => return Err(Failure::no_step(id)),
            Some(id) => View::Route(
                Trail::new(self.trails()?.steps().find(id)?)
                    .work()
                    .id()
                    .to_string(),
            ),
            None => View::Cards,
        };
        let width = self.columns().map_or(WIDTH, |c| c.saturating_sub(1));
        let waiting = empty(&weekday(crate::command::now()), width);
        self.live(out, waiting, |_| {
            Ok(Board {
                command: self,
                view,
                picked: 0,
                row: 0,
                back: vec![],
                per_row: 0,
                kept: None,
                head: None,
                rows: vec![],
                drawn: None,
            })
        })
    }
}

impl Live for Board<'_> {
    fn renew(&mut self, _: &Model, _: Option<&[Note]>) {
        (self.kept, self.head) = (None, None);
    }

    /// Its page of `model`, drawn again each minute; after a minute it forgets its cards and status.
    fn page(&mut self, model: &Model, _: &mut Screen) -> Result<Page, Failure> {
        if self.drawn.is_some_and(|at| at.elapsed() >= MINUTE) {
            (self.kept, self.head) = (None, None);
        }
        self.drawn = Some(Instant::now());
        let width = self
            .command
            .columns()
            .map_or(WIDTH, |c| c.saturating_sub(1));
        Ok(Page {
            again: Some(MINUTE),
            ..self.shown(model, width)
        })
    }

    /// Moves on the page it last drew.
    fn press(&mut self, key: Key, _: &Model, screen: &mut Screen) -> Result<(), Failure> {
        self.go(key);
        screen.follow = true;
        Ok(())
    }
}

/// The time a board waits before it draws its page again.
const MINUTE: Duration = Duration::from_secs(60);

/// The width a board draws at when the terminal's is not known.
const WIDTH: usize = 80;

/// The width below which a board draws its numbers as plain text.
const NARROW: usize = 60;

/// Where a board stands: what it shows, the picked card, the selected row, the routes it came from,
/// the cards a row it last drew, the cards it last made with the sizes it made them for, whether a step
/// closed with its status line, the id of each row it last drew, and when it drew last.
struct Board<'a> {
    command: &'a Command,
    view: View,
    picked: usize,
    row: usize,
    back: Vec<String>,
    per_row: usize,
    kept: Option<((usize, usize, bool), [Sheet; CARDS])>,
    head: Option<(bool, String)>,
    rows: Vec<Option<String>>,
    drawn: Option<Instant>,
}

/// What a board shows.
enum View {
    /// The empty card of a store with no closed step.
    Empty,
    /// Every card, the picked one framed.
    Cards,
    /// The picked card opened to its table.
    Card,
    /// The route of the step with this id.
    Route(String),
}

impl Board<'_> {
    /// The page of `model` it shows, `width` wide, from the cards and status it kept when their sizes
    /// are the same.
    fn shown(&mut self, model: &Model, width: usize) -> Page {
        self.rows = vec![];
        let steps = model.steps;
        if let View::Route(id) = &self.view {
            if let Some(step) = steps.get(id) {
                return self.route(model, step, width);
            }
        }
        let (closed, status) = self
            .head
            .get_or_insert_with(|| (steps.closed_any(), status(steps)))
            .clone();
        if !closed {
            self.view = View::Empty;
            return empty(&status, width);
        }
        if let View::Empty = self.view {
            self.view = View::Cards;
        }
        self.per_row = match width {
            w if w >= 120 => 3,
            w if w >= 80 => 2,
            _ => 1,
        };
        let card = (width - (self.per_row - 1)) / self.per_row;
        let full = card * self.per_row + self.per_row - 1;
        let sizes = (full - 4, card - 4, width < NARROW);
        let (sizes, sheets) = match self.kept.take() {
            Some((kept, sheets)) if kept == sizes => (kept, sheets),
            _ => (sizes, sheets(model, sizes.0, sizes.1, sizes.2)),
        };
        self.picked = self.picked.min(CARDS - 1);
        let page = match self.view {
            View::Card => self.opened(&sheets[self.picked], width),
            _ => self.cards(&status, &sheets, (full, card), width),
        };
        self.kept = Some((sizes, sheets));
        page
    }

    /// The page of each card under `status`: the effect across `widths.0`, the others `widths.1` wide.
    fn cards(
        &self,
        status: &str,
        sheets: &[Sheet; CARDS],
        (full, card): (usize, usize),
        width: usize,
    ) -> Page {
        let mut lines = head(None, status, width);
        let markdown = sheets[self.picked]
            .table
            .markdown(&sheets[self.picked].title);
        let cards: Vec<Card> = sheets
            .iter()
            .enumerate()
            .map(|(i, s)| Card {
                title: s.title.clone(),
                rows: s.face.clone(),
                picked: i == self.picked,
            })
            .collect();
        let mut line = None;
        let first = &cards[0];
        if self.picked == 0 {
            line = Some(lines.len());
        }
        lines.extend(first.lines(full, first.rows.len() + 2));
        for (r, row) in cards[1..].chunks(self.per_row).enumerate() {
            if self.picked > 0 && (self.picked - 1) / self.per_row == r {
                line = Some(lines.len());
            }
            lines.extend(grid(row, self.per_row, card));
        }
        Page {
            lines,
            picked: line,
            keys: "←→↑↓ move · enter open · m copy markdown · q quit".into(),
            markdown: Some(markdown),
            ..Page::default()
        }
    }

    /// The page of the opened card `sheet`: its table, a row a line.
    fn opened(&mut self, sheet: &Sheet, width: usize) -> Page {
        let ids: Vec<Option<String>> = sheet.table.rows.iter().map(|r| r.id.clone()).collect();
        self.row = self.row.min(ids.len().saturating_sub(1));
        let mut lines = head(Some(&sheet.title), "", width);
        lines.extend(sheet.table.lines(width, Some(self.row)));
        let copy = ids.get(self.row).cloned().flatten();
        self.rows = ids;
        Page {
            lines,
            picked: Some(3 + self.row),
            keys: legend("↑↓ select · ← back", copy.as_deref()),
            copy: copy.map(|id| short(&id)),
            markdown: Some(sheet.table.markdown(&sheet.title)),
            ..Page::default()
        }
    }

    /// The page of the route of `step`: the steps it follows, the step, and the steps that follow it.
    fn route(&mut self, model: &Model, step: Step, width: usize) -> Page {
        let reuse = model.reuse;
        let up = reuse.up(step);
        let down = reuse.down(step);
        let count = up.len() + down.len();
        self.row = self.row.min(count.saturating_sub(1));
        let name = &step.record().anchor.name;
        let mut lines = head(Some(&short(step.id())), name, width);
        let mut rows = vec![];
        let mut table = vec![];
        let mut line = None;
        let mut item = |lines: &mut Vec<Styled>, kind: &str, at: Step, depth: usize| {
            let look = look(at.state());
            let tail = time(at).unwrap_or_else(|| at.state().text().to_string());
            let lead = "  ".repeat(depth.saturating_sub(1));
            let room = width.saturating_sub(visible(&tail) + lead.len() + 8);
            let name = cut(at.title(), room);
            let pad = " ".repeat(room.saturating_sub(visible(&name)));
            let picked = rows.len() == self.row;
            if picked {
                line = Some(lines.len());
            }
            let spans = vec![
                Span::raw(match picked {
                    true => format!("▌ {lead}"),
                    false => format!("  {lead}"),
                }),
                paint(look.text, look.mark),
                Span::raw(format!(" {name}{pad}  ")),
                paint(DIM, &tail),
            ];
            lines.push(Styled::from(spans));
            rows.push(Some(at.id().to_string()));
            table.push(Row {
                id: Some(at.id().to_string()),
                cells: vec![
                    kind.to_string(),
                    at.title().to_string(),
                    at.record().anchor.name.clone(),
                    time(at).unwrap_or("—".into()),
                ],
            });
        };
        if !up.is_empty() {
            lines.push(paint(DIM, "  FOLLOWS").into());
            for link in &up {
                item(&mut lines, "follows", link.source, link.depth);
            }
            lines.push(Styled::default());
        }
        let ink = Ink {
            plain: width < NARROW,
        };
        let took = time(step).unwrap_or("—".into());
        let side = [
            ink.dim("took").into(),
            ink.dim(&format!("revised {}", step.revised().len())).into(),
        ];
        let face = ink.beside(&took, &side, 3);
        let look = look(step.state());
        let card = Card {
            title: format!("{} {}", look.mark, step.title()),
            rows: face,
            picked: true,
        };
        lines.extend(card.lines(width, card.rows.len() + 2));
        if !down.is_empty() {
            let followed = paint(DIM, "  FOLLOWED BY").into();
            lines.extend([Styled::default(), followed]);
            for link in &down {
                item(&mut lines, "followed by", link.step, link.depth);
            }
        }
        if count == 0 {
            lines.extend([
                Styled::default(),
                paint(DIM, "  it follows no step, and no step follows it").into(),
            ]);
        }
        let copy = rows.get(self.row).cloned().flatten();
        let markdown = Table {
            head: vec!["Link", "Step", "Project", "Took"],
            rows: table,
            wide: 1,
        }
        .markdown(&format!("{} {}", short(step.id()), step.title()));
        self.rows = rows;
        Page {
            lines,
            picked: line,
            keys: legend("↑↓ select · enter go to · ← back", copy.as_deref()),
            copy: copy.map(|id| short(&id)),
            markdown: Some(markdown),
            ..Page::default()
        }
    }

    /// Moves as `key` asks on `page`: among the cards, the rows of an opened card, or along a route.
    fn go(&mut self, key: Key) {
        let last = self.rows.len().saturating_sub(1);
        match (&self.view, key) {
            (View::Empty, _) => {}
            (View::Card | View::Route(_), Key::Up) => self.row = self.row.saturating_sub(1),
            (View::Card | View::Route(_), Key::Down) => self.row = (self.row + 1).min(last),
            (View::Card, Key::Left | Key::Enter) => self.view = View::Cards,
            (View::Route(from), Key::Enter | Key::Right) => {
                if let Some(Some(id)) = self.rows.get(self.row).cloned() {
                    self.back.push(from.clone());
                    self.view = View::Route(id.clone());
                    self.row = 0;
                }
            }
            (View::Route(_), Key::Left) => {
                if let Some(id) = self.back.pop() {
                    self.view = View::Route(id);
                    self.row = 0;
                }
            }
            (View::Cards, Key::Enter) => {
                self.view = View::Card;
                self.row = 0;
            }
            (View::Cards, key) => self.picked = self.next(key),
            _ => {}
        }
    }

    /// The card `key` picks from the picked one, among the effect across the width and the rest a row.
    fn next(&self, key: Key) -> usize {
        let per_row = self.per_row.max(1);
        let picked = match key {
            Key::Left => self.picked.saturating_sub(1),
            Key::Right => self.picked + 1,
            Key::Down if self.picked == 0 => 1,
            Key::Down => self.picked + per_row,
            Key::Up if self.picked <= per_row => 0,
            Key::Up => self.picked - per_row,
            _ => self.picked,
        };
        picked.min(CARDS - 1)
    }
}

/// The page of a store with no closed step, `right` in its header: a card that says how stats fill.
fn empty(right: &str, width: usize) -> Page {
    let card = Card {
        title: "NO STATS YET".into(),
        rows: vec![
            "Stats show when your agent finishes its first step.".into(),
            Styled::from(vec![
                "Run ".into(),
                paint(BOLD, "celis stats --demo"),
                " to see sample data.".into(),
            ]),
        ],
        picked: false,
    };
    let mut lines = head(None, right, width);
    lines.extend(card.lines(width, card.rows.len() + 2));
    Page {
        lines,
        keys: "q quit".into(),
        ..Page::default()
    }
}

/// `● <n> active · <weekday>` of `steps`.
fn status(steps: &Steps) -> String {
    let active = steps
        .roots()
        .into_iter()
        .filter(|s| s.state().active())
        .count();
    format!("● {active} active · {}", weekday(steps.now()))
}

/// The head of a page, `width` wide: ` CELIS STATS[ › <place>]` bold and `right` dim at the two ends,
/// then a blank line.
fn head(place: Option<&str>, right: &str, width: usize) -> Vec<Styled> {
    let left = place.map_or(" CELIS STATS".into(), |p| format!(" CELIS STATS › {p}"));
    let pad = width.saturating_sub(visible(&left) + visible(right) + 1);
    let line = Styled::from(vec![
        paint(BOLD, &left),
        " ".repeat(pad).into(),
        paint(DIM, right),
    ]);
    vec![line, Styled::default()]
}

/// The key line: `keys`, then `c copy <id>` for the id `copy` names, then the copy and quit keys.
fn legend(keys: &str, copy: Option<&str>) -> String {
    let copy = copy.map_or(String::new(), |id| format!(" · c copy {}", short(id)));
    format!("{keys}{copy} · m copy markdown · q quit")
}
