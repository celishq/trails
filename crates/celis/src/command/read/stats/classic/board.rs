//! `celis stats` on a terminal: its cards, an opened card, or the route of a step, drawn again each
//! time a record lands in the store or a linked folder.

use super::card::{cards, Card, CARDS};
use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::progress::Model;
use crate::terminal::canvas::visible;
use crate::terminal::canvas::{paint, Styled, BOLD, DIM};
use crate::terminal::live::{Live, Page};
use crate::terminal::screen::Screen;
use crate::terminal::wake::Key;
use crate::text::clock::weekday;
use cards::{empty, status};
use celis_nostr::Note;
use celis_trails::records::id::short;
use celis_trails::Trail;
use std::io::Write;
use std::time::{Duration, Instant};

impl Command {
    /// Draws the cards of stats on the alternate screen, or the route of the step `args` names, then
    /// again when a record lands, a key is read, a minute passes or the terminal's size changes. With
    /// no store, it writes nothing and waits for one. On an interrupt or `q`, leaves the alternate
    /// screen with nothing printed, and ends.
    pub(crate) fn cards(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
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
    kept: Option<((usize, usize, bool), [Card; CARDS])>,
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
        let (sizes, drawn) = match self.kept.take() {
            Some((kept, drawn)) if kept == sizes => (kept, drawn),
            _ => (sizes, cards(model, sizes.0, sizes.1, sizes.2)),
        };
        self.picked = self.picked.min(CARDS - 1);
        let page = match self.view {
            View::Card => self.opened(&drawn[self.picked], width),
            _ => self.cards(&status, &drawn, (full, card), width),
        };
        self.kept = Some((sizes, drawn));
        page
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

mod cards;
mod route;
