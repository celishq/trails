//! `celis stats` on a terminal: its cards, an opened card, or the route of a step, drawn again each
//! time a record lands in the store or a linked folder.

use super::card::{grid, paint, Card};
use super::clock::{saved, took, weekday};
use super::screen::{clipboard, Screen};
use super::sheet::{sheets, Sheet};
use super::table::{Row, Table};
use super::tree::{cut, look, visible};
use super::wake::{Key, Wake};
use super::{Args, Command};
use celis_trails::id::short;
use celis_trails::{Failure, Period, Reuse, Step, Trail, Trails};
use std::io::Write;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

impl Command {
    /// Draws the cards of stats on the alternate screen, or the route of the step `args` names, then
    /// again when a record lands, a key is read, a minute passes or the terminal's size changes. With
    /// no store, it writes nothing and waits for one. On an interrupt or `q`, leaves the alternate
    /// screen with nothing printed, and ends.
    pub(super) fn board(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
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
        let board = Board {
            route,
            ..Board::default()
        };
        self.screen(out, |out, wakes| self.boarded(board, out, wakes))
    }

    /// The board's loop: draws each page it reads from the store, until an interrupt.
    fn boarded(
        &self,
        mut board: Board,
        out: &mut dyn Write,
        (send, events): (Sender<Wake>, Receiver<Wake>),
    ) -> Result<(), Failure> {
        let mut screen = Screen::default();
        let width = || self.columns().map_or(WIDTH, |c| c.saturating_sub(1));
        if !self.held()? {
            let page = empty(&weekday(super::now()), width(), self.colour());
            screen.draw(out, &page.text, page.line, &page.legend)?;
            while !self.held()? {
                if let Ok(Wake::Quit) = events.recv_timeout(Duration::from_millis(250)) {
                    return Ok(());
                }
            }
        }
        let mut trails = self.trails()?;
        let _watcher = self.listen(send)?;
        loop {
            trails.now = super::now();
            let page = board.page(&trails, width(), self.colour());
            screen.draw(out, &page.text, page.line, &page.legend)?;
            let mut wake = events.recv_timeout(Duration::from_secs(60));
            let (mut landed, mut linked) = (false, false);
            loop {
                match wake {
                    Ok(Wake::Record(from)) => (landed, linked) = (true, linked || from),
                    Ok(Wake::Resize) => screen.shown.clear(),
                    Ok(Wake::Key(key)) => {
                        screen.said = match (&key, &page.copy, &page.markdown) {
                            (Key::Copy, Some(id), _) => {
                                Some(clipboard(&short(id), &format!("copied {}", short(id))))
                            }
                            (Key::Markdown, _, Some(text)) => {
                                Some(clipboard(text, "copied markdown"))
                            }
                            _ => None,
                        };
                        screen.press(&key);
                        board.press(key, &page);
                        screen.follow = true;
                    }
                    Ok(Wake::Quit) | Err(RecvTimeoutError::Disconnected) => return Ok(()),
                    Err(RecvTimeoutError::Timeout) => break,
                }
                if !landed {
                    break;
                }
                wake = events.recv_timeout(Duration::from_millis(50));
            }
            match (landed, linked) {
                (_, true) => trails = self.trails()?,
                (true, false) => _ = trails.fresh(),
                _ => {}
            }
        }
    }
}

/// The width a board draws at when the terminal's is not known.
const WIDTH: usize = 80;

/// The width below which a board draws its numbers as plain text.
const NARROW: usize = 60;

/// Where a board stands: the picked card, whether it is opened, the selected row, the step whose route
/// it shows, the routes it came from, and the cards a row it last drew.
#[derive(Default)]
struct Board {
    picked: usize,
    opened: bool,
    row: usize,
    route: Option<String>,
    back: Vec<String>,
    per_row: usize,
}

/// One drawn page: its text, the line to keep in view, its key line, the id `c` copies, the Markdown
/// `m` copies, and the id of each row it selects.
struct Page {
    text: String,
    line: Option<usize>,
    legend: String,
    copy: Option<String>,
    markdown: Option<String>,
    rows: Vec<Option<String>>,
}

impl Board {
    /// The page of `trails` it shows, `width` wide.
    fn page(&mut self, trails: &Trails, width: usize, colour: bool) -> Page {
        let steps = trails.steps();
        if let Some(step) = self.route.as_deref().and_then(|id| steps.get(id)) {
            return self.route(trails, step, width, colour);
        }
        if Period::new(&steps, 0, u64::MAX).closed().is_empty() {
            return empty(&status(trails), width, colour);
        }
        self.per_row = match width {
            w if w >= 120 => 3,
            w if w >= 80 => 2,
            _ => 1,
        };
        let card = (width - (self.per_row - 1)) / self.per_row;
        let full = card * self.per_row + self.per_row - 1;
        let sheets = sheets(trails, full - 4, card - 4, width < NARROW, colour);
        self.picked = self.picked.min(sheets.len() - 1);
        match self.opened {
            true => self.opened(&sheets[self.picked], width, colour),
            false => self.cards(trails, sheets, (full, card), width, colour),
        }
    }

    /// The page of each card: the effect across `widths.0`, the others `widths.1` wide.
    fn cards(
        &self,
        trails: &Trails,
        sheets: Vec<Sheet>,
        (full, card): (usize, usize),
        width: usize,
        colour: bool,
    ) -> Page {
        let mut lines = vec![
            header(" CELIS STATS", &status(trails), width, colour),
            String::new(),
        ];
        let markdown = sheets[self.picked]
            .table
            .markdown(&sheets[self.picked].title);
        let cards: Vec<Card> = sheets
            .into_iter()
            .enumerate()
            .map(|(i, s)| Card {
                title: s.title,
                rows: s.face,
                picked: i == self.picked,
            })
            .collect();
        let mut line = None;
        let first = &cards[0];
        if self.picked == 0 {
            line = Some(lines.len());
        }
        lines.extend(first.lines(full, first.rows.len() + 2, colour));
        for (r, row) in cards[1..].chunks(self.per_row).enumerate() {
            if self.picked > 0 && (self.picked - 1) / self.per_row == r {
                line = Some(lines.len());
            }
            lines.extend(grid(row, self.per_row, card, colour));
        }
        Page {
            text: lines.join("\n"),
            line,
            legend: "←→↑↓ move · enter open · m copy markdown · q quit".into(),
            copy: None,
            markdown: Some(markdown),
            rows: vec![],
        }
    }

    /// The page of the opened card `sheet`: its table, a row a line.
    fn opened(&mut self, sheet: &Sheet, width: usize, colour: bool) -> Page {
        let ids: Vec<Option<String>> = sheet.table.rows.iter().map(|r| r.id.clone()).collect();
        self.row = self.row.min(ids.len().saturating_sub(1));
        let title = format!(" CELIS STATS › {}", sheet.title);
        let mut lines = vec![header(&title, "", width, colour), String::new()];
        lines.extend(sheet.table.lines(width, Some(self.row), colour));
        let copy = ids.get(self.row).cloned().flatten();
        Page {
            text: lines.join("\n"),
            line: Some(3 + self.row),
            legend: legend("↑↓ select · ← back", copy.as_deref()),
            copy,
            markdown: Some(sheet.table.markdown(&sheet.title)),
            rows: ids,
        }
    }

    /// The page of the route of `step`: the steps it follows, the step, and the steps that follow it.
    fn route(&mut self, trails: &Trails, step: Step, width: usize, colour: bool) -> Page {
        let steps = trails.steps();
        let reuse = Reuse::new(&steps);
        let up = reuse.up(step);
        let down = reuse.down(step);
        let count = up.len() + down.len();
        self.row = self.row.min(count.saturating_sub(1));
        let title = format!(" CELIS STATS › {}", short(step.id()));
        let mut lines = vec![
            header(&title, &step.record().anchor.name, width, colour),
            String::new(),
        ];
        let mut rows = vec![];
        let mut table = vec![];
        let mut line = None;
        let mut item =
            |lines: &mut Vec<String>, kind: &str, at: Step, depth: usize, gain: Option<i64>| {
                let look = look(at.state());
                let time = at
                    .grade()
                    .map_or(at.state().text().to_string(), |g| took(g.took));
                let gain_text = gain.map_or(String::new(), |g| format!("  saved {}", saved(g)));
                let tail = format!("{time}{gain_text}");
                let lead = "  ".repeat(depth.saturating_sub(1));
                let room = width.saturating_sub(visible(&tail) + lead.len() + 8);
                let name = cut(at.title(), room);
                let pad = " ".repeat(room.saturating_sub(visible(&name)));
                let text = format!(
                    "{lead}{} {name}{pad}  {}",
                    paint(colour, look.text, look.mark),
                    paint(colour, "2", &tail)
                );
                let picked = rows.len() == self.row;
                if picked {
                    line = Some(lines.len());
                }
                lines.push(match picked {
                    true => format!("▌ {text}"),
                    false => format!("  {text}"),
                });
                rows.push(Some(at.id().to_string()));
                table.push(Row {
                    id: Some(at.id().to_string()),
                    cells: vec![
                        kind.to_string(),
                        at.title().to_string(),
                        at.record().anchor.name.clone(),
                        at.grade().map_or("—".into(), |g| took(g.took)),
                        gain.map_or("—".into(), saved),
                    ],
                });
            };
        if !up.is_empty() {
            lines.push(paint(colour, "2", "  FOLLOWS"));
            for link in &up {
                item(&mut lines, "follows", link.source, link.depth, None);
            }
            lines.push(String::new());
        }
        let time = step.grade().map(|g| took(g.took));
        let big = match width < NARROW {
            true => vec![paint(colour, "1", time.as_deref().unwrap_or("—"))],
            false => super::glyph::digits(time.as_deref().unwrap_or("—"))
                .iter()
                .map(|r| paint(colour, "1", r))
                .collect(),
        };
        let revised = step.revised().len();
        let side = [
            paint(colour, "2", "took"),
            paint(colour, "2", &format!("revised {revised}")),
        ];
        let room = big.iter().map(|r| visible(r)).max().unwrap_or(0) + 3;
        let face: Vec<String> = big
            .iter()
            .enumerate()
            .map(|(i, r)| {
                format!(
                    "{r}{}{}",
                    " ".repeat(room - visible(r)),
                    side.get(i).map_or("", String::as_str)
                )
            })
            .collect();
        let look = look(step.state());
        let card = Card {
            title: format!("{} {}", look.mark, step.title()),
            rows: face,
            picked: true,
        };
        lines.extend(card.lines(width, card.rows.len() + 2, colour));
        if !down.is_empty() {
            lines.extend([String::new(), paint(colour, "2", "  FOLLOWED BY")]);
            for link in &down {
                item(
                    &mut lines,
                    "followed by",
                    link.step,
                    link.depth,
                    reuse.saving(link),
                );
            }
            if let Some(total) = reuse.saved(step) {
                lines.extend([
                    String::new(),
                    format!("  saved in total  {}", paint(colour, "1", &saved(total))),
                ]);
            }
        }
        if count == 0 {
            lines.extend([
                String::new(),
                paint(colour, "2", "  it follows no step, and no step follows it"),
            ]);
        }
        let copy = rows.get(self.row).cloned().flatten();
        let markdown = Table {
            head: vec!["Link", "Step", "Project", "Took", "Saved"],
            rows: table,
            wide: 1,
        }
        .markdown(&format!("{} {}", short(step.id()), step.title()));
        Page {
            text: lines.join("\n"),
            line,
            legend: legend("↑↓ select · enter go to · ← back", copy.as_deref()),
            copy,
            markdown: Some(markdown),
            rows,
        }
    }

    /// Moves as `key` asks on `page`: among the cards, the rows of an opened card, or along a route;
    /// nothing on a page with no Markdown, as the empty page.
    fn press(&mut self, key: Key, page: &Page) {
        if page.markdown.is_none() {
            return;
        }
        let last = page.rows.len().saturating_sub(1);
        if self.route.is_some() {
            match key {
                Key::Up => self.row = self.row.saturating_sub(1),
                Key::Down => self.row = (self.row + 1).min(last),
                Key::Enter | Key::Right => {
                    if let Some(Some(id)) = page.rows.get(self.row) {
                        let from = self.route.replace(id.clone());
                        self.back.extend(from);
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
            return;
        }
        if self.opened {
            match key {
                Key::Up => self.row = self.row.saturating_sub(1),
                Key::Down => self.row = (self.row + 1).min(last),
                Key::Left | Key::Enter => self.opened = false,
                _ => {}
            }
            return;
        }
        let per_row = self.per_row.max(1);
        self.picked = match key {
            Key::Left => self.picked.saturating_sub(1),
            Key::Right => self.picked + 1,
            Key::Down if self.picked == 0 => 1,
            Key::Down => self.picked + per_row,
            Key::Up if self.picked <= per_row => 0,
            Key::Up => self.picked - per_row,
            _ => self.picked,
        }
        .min(6);
        if let Key::Enter = key {
            self.opened = true;
            self.row = 0;
        }
    }
}

/// The page of a store with no closed step, `right` in its header: a card that says how stats fill.
fn empty(right: &str, width: usize, colour: bool) -> Page {
    let card = Card {
        title: "NO STATS YET".into(),
        rows: vec![
            "Stats show when your agent finishes its first step.".into(),
            format!(
                "Run {} to see sample data.",
                paint(colour, "1", "celis stats --demo")
            ),
        ],
        picked: false,
    };
    let mut lines = vec![header(" CELIS STATS", right, width, colour), String::new()];
    lines.extend(card.lines(width, card.rows.len() + 2, colour));
    Page {
        text: lines.join("\n"),
        line: None,
        legend: "q quit".into(),
        copy: None,
        markdown: None,
        rows: vec![],
    }
}

/// `● <n> active · <weekday>` of `trails`.
fn status(trails: &Trails) -> String {
    let active = trails
        .steps()
        .roots()
        .into_iter()
        .filter(|s| s.state().active())
        .count();
    format!("● {active} active · {}", weekday(trails.now))
}

/// `left` bold and `right` dim at the two ends of a line `width` wide.
fn header(left: &str, right: &str, width: usize, colour: bool) -> String {
    let pad = width.saturating_sub(visible(left) + visible(right) + 1);
    format!(
        "{}{}{}",
        paint(colour, "1", left),
        " ".repeat(pad),
        paint(colour, "2", right)
    )
}

/// The key line: `keys`, then `c copy <id>` for the id `copy` names, then the copy and quit keys.
fn legend(keys: &str, copy: Option<&str>) -> String {
    let copy = copy.map_or(String::new(), |id| format!(" · c copy {}", short(id)));
    format!("{keys}{copy} · m copy markdown · q quit")
}
