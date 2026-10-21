//! `celis stats` on a terminal: the last 14 days against the prior 14 as headline tiles and sections, or
//! the route of a step, drawn again each time a record lands, a key is read or the board moves.

use super::head::{group, head, headline, total};
use super::moment::Moment;
use super::month::{Month, DAYS};
use super::section::Section;
use super::sheet::Sheet;
use super::total::Total;
use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::progress::Model;
use crate::terminal::canvas::visible;
use crate::terminal::canvas::{paint, Styled, BOLD, DIM};
use crate::terminal::live::{Live, Page};
use crate::terminal::screen::Screen;
use crate::terminal::wake::Key;
use celis_nostr::Note;
use celis_trails::records::id::short;
use celis_trails::Trail;
use std::io::Write;
use std::time::{Duration, Instant};

impl Command {
    /// Draws the board of stats on the alternate screen, or the route of the step `args` names, then
    /// again when a record lands, a key is read, the board moves or a minute passes. With no store, it
    /// writes nothing and waits for one. On an interrupt or `q`, leaves the alternate screen with
    /// nothing printed, and ends.
    pub(crate) fn board(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let route = match args.words.get(1) {
            Some(id) if !self.held()? => return Err(Failure::no_step(id)),
            Some(id) => Some(
                Trail::new(self.trails()?.steps().find(id)?)
                    .work()
                    .id()
                    .to_string(),
            ),
            None => None,
        };
        let waiting = empty(breadth(self));
        self.live(out, waiting, |_| Ok(Board::new(self, route)))
    }
}

/// The time a board waits before it reads its month again.
const MINUTE: Duration = Duration::from_secs(60);

/// The seconds the board takes to arrive whole.
const ARRIVAL: f64 = 2.6;

/// The narrowest and widest a board draws.
const WIDTHS: (u16, u16) = (32, 120);

/// The width below which a route draws its numbers as plain text.
const NARROW: usize = 60;

/// Where a board stands: the step whose route it shows with the selected row, the routes it came from
/// and the id of each row it last drew; the month it read and when; when it opened; the section
/// picked, the sections opened, the day picked, the first row shown, whether it scrolls to the
/// section picked, the rows each section took in the last frame, the rows of that frame, and the rows
/// the screen shows.
struct Board<'a> {
    command: &'a Command,
    route: Option<String>,
    row: usize,
    back: Vec<String>,
    rows: Vec<Option<String>>,
    month: Option<(Month, Total, Instant)>,
    opened: Instant,
    focus: usize,
    open: [bool; 4],
    day: usize,
    top: u16,
    follow: bool,
    places: Vec<(u16, u16)>,
    length: u16,
    room: u16,
}

impl<'a> Board<'a> {
    fn new(command: &'a Command, route: Option<String>) -> Board<'a> {
        Board {
            command,
            route,
            row: 0,
            back: vec![],
            rows: vec![],
            month: None,
            opened: Instant::now(),
            focus: 0,
            open: [false; 4],
            day: DAYS - 1,
            top: 0,
            follow: false,
            places: vec![],
            length: 0,
            room: 40,
        }
    }

    /// The key line: the sections, opening the picked one or closing it, all, the day while Output and
    /// speed is open, and quit.
    fn keys(&self) -> String {
        let enter = if self.open[self.focus] {
            "close"
        } else {
            "open"
        };
        let mut keys = format!("↑↓ section · enter {enter} · a all");
        if self.open[2] {
            keys.push_str(" · ←→ day");
        }
        keys + " · q quit"
    }

    /// Moves as `key` asks: ↑↓ scroll through a section taller than the screen, then pick the section
    /// before or after; enter opens or closes the picked one, `a` all; ←→ pick a day.
    fn go(&mut self, key: Key) {
        let (first, last) = self.places.get(self.focus).copied().unwrap_or((0, 0));
        let room = self.room.max(2);
        match key {
            Key::Up if first < self.top => self.top = self.top.saturating_sub(3).max(first),
            Key::Up if self.focus == 0 => self.top = self.top.saturating_sub(3),
            Key::Up => (self.focus, self.follow) = (self.focus - 1, true),
            Key::Down if last > self.top + room => {
                self.top = (last - (room - 1)).min(self.top + 3);
            }
            Key::Down if self.focus + 1 < Section::ALL.len() => {
                (self.focus, self.follow) = (self.focus + 1, true);
            }
            Key::Enter | Key::Space => {
                self.open[self.focus] = !self.open[self.focus];
                self.follow = true;
            }
            Key::All => {
                let every = self.open.iter().all(|o| *o);
                self.open = [!every; 4];
                self.follow = true;
            }
            Key::Right => self.day = (self.day + 1).min(DAYS - 1),
            Key::Left => self.day = self.day.saturating_sub(1),
            Key::PageUp => self.top = self.top.saturating_sub(room - 2),
            Key::PageDown => self.top += room - 2,
            _ => {}
        }
    }

    /// Moves along a route as `key` asks: ↑↓ select a row, enter or → goes to its step, ← back.
    fn walk(&mut self, key: Key) {
        let last = self.rows.len().saturating_sub(1);
        match key {
            Key::Up => self.row = self.row.saturating_sub(1),
            Key::Down => self.row = (self.row + 1).min(last),
            Key::Enter | Key::Right => {
                if let (Some(Some(id)), Some(from)) =
                    (self.rows.get(self.row).cloned(), &self.route)
                {
                    self.back.push(from.clone());
                    self.route = Some(id);
                    self.row = 0;
                }
            }
            Key::Left => {
                if let Some(id) = self.back.pop() {
                    self.route = Some(id);
                    self.row = 0;
                }
            }
            _ => {}
        }
    }
}

impl Live for Board<'_> {
    fn renew(&mut self, _: &Model, _: Option<&[Note]>) {
        self.month = None;
    }

    /// The page of `model`: the route it shows, the empty board of a store with no closed step, or the
    /// rows of the board the screen holds, from the first one shown; drawn again each frame while it
    /// arrives, each pulse while a trail is active, else each minute.
    fn page(&mut self, model: &Model, _: &mut Screen) -> Result<Page, Failure> {
        let width = breadth(self.command);
        if let Some(id) = self.route.clone() {
            if let Some(step) = model.steps.get(&id) {
                return Ok(self.route(model, step, usize::from(width)));
            }
        }
        if !model.steps.closed_any() {
            return Ok(empty(width));
        }
        if self
            .month
            .as_ref()
            .is_some_and(|(_, _, at)| at.elapsed() >= MINUTE)
        {
            self.month = None;
        }
        let (month, total, _) = self
            .month
            .get_or_insert_with(|| (Month::of(model), Total::of(model), Instant::now()));
        let size = crate::command::size();
        self.room = size.map_or(40, |s| s.ws_row.saturating_sub(1)).max(4);
        let t = self.opened.elapsed().as_secs_f64();
        let m = Moment {
            month,
            total,
            t,
            width,
            day: self.day,
            focus: self.focus,
            open: self.open,
        };
        let (sheet, length, places) = frame(&m);
        let active = month.active > 0;
        self.places = places;
        self.length = length;
        if self.follow {
            let (first, last) = self.places[self.focus];
            if last.saturating_sub(self.top) > self.room - 1 {
                self.top = first.min(last - (self.room - 1));
            }
            self.top = self.top.min(first);
            self.follow = false;
        }
        self.top = self.top.min(self.length.saturating_sub(self.room));
        let again = match (t < ARRIVAL, active) {
            (true, _) => Duration::from_millis(16),
            (false, true) => Duration::from_millis(33),
            (false, false) => MINUTE,
        };
        Ok(Page {
            lines: sheet.lines(self.top, self.top + self.room),
            keys: self.keys(),
            again: Some(again),
            ..Page::default()
        })
    }

    fn press(&mut self, key: Key, _: &Model, screen: &mut Screen) -> Result<(), Failure> {
        match self.route {
            Some(_) => {
                self.walk(key);
                screen.follow = true;
            }
            None => self.go(key),
        }
        Ok(())
    }
}

/// The board of `m` on a sheet: the head, the Total line, the overview with its headline tiles, then
/// each section
/// closed or opened; the rows it takes, and the first row of each section with the row after it.
fn frame(m: &Moment) -> (Sheet, u16, Vec<(u16, u16)>) {
    let mut s = Sheet::new(m.width, 64);
    head(&mut s, m);
    let y = total(&mut s, m, 2);
    let mut y = group(&mut s, m, y, "Overview", "last 14 days vs prior 14");
    y = headline(&mut s, m, y);
    let mut places = vec![];
    for (n, section) in Section::ALL.into_iter().enumerate() {
        let top = y;
        y = section.draw(&mut s, m, y, n);
        places.push((top, y));
    }
    (s, y + 1, places)
}

/// The width the board of `command` draws at: the terminal's less a column, held between the
/// narrowest and widest, the widest off a terminal.
fn breadth(command: &Command) -> u16 {
    let columns = command.columns().map_or(usize::from(WIDTHS.1) + 1, |c| c);
    let columns = u16::try_from(columns.saturating_sub(1)).unwrap_or(WIDTHS.1);
    columns.clamp(WIDTHS.0, WIDTHS.1)
}

/// The page of a store with no closed step, `width` wide: a card that says how stats fill.
fn empty(width: u16) -> Page {
    let mut s = Sheet::new(width, 8);
    s.put(2, 0, "CELIS STATS", BOLD);
    let w = width.saturating_sub(4).min(64);
    s.card((2, 2, w, 5), "No stats yet", "");
    s.put(
        4,
        3,
        "Stats show when your agent finishes its first step.",
        Default::default(),
    );
    let run = [
        paint(Default::default(), "Run "),
        paint(BOLD, "celis stats --demo"),
        paint(Default::default(), " to see sample data."),
    ];
    s.spans(4, 4, &run);
    Page {
        lines: s.lines(0, 7),
        keys: "q quit".into(),
        ..Page::default()
    }
}

/// The head of a route, `width` wide: ` CELIS STATS › <place>` bold and `right` dim at the two ends,
/// then a blank line.
fn route_head(place: &str, right: &str, width: usize) -> Vec<Styled> {
    let left = format!(" CELIS STATS › {place}");
    let pad = width.saturating_sub(visible(&left) + visible(right) + 1);
    let line = Styled::from(vec![
        paint(BOLD, &left),
        " ".repeat(pad).into(),
        paint(DIM, right),
    ]);
    vec![line, Styled::default()]
}

/// The key line of a route: `keys`, then `c copy <id>` for the id `copy` names, then the copy and quit
/// keys.
fn legend(keys: &str, copy: Option<&str>) -> String {
    let copy = copy.map_or(String::new(), |id| format!(" · c copy {}", short(id)));
    format!("{keys}{copy} · m copy markdown · q quit")
}

mod route;
#[cfg(test)]
mod tests;
