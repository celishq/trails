//! A page drawn live from a store: again each time a record lands, a key is read, or its time to draw
//! again comes.

use super::canvas::Styled;
use super::look::{spin, spinner, DRAWN, TURN};
use super::remark::{self, Remark};
use super::scene::{self, scene, FADE, PLAYS};
use super::screen::{clipboard, Screen};
use super::unfold::{fade, length, smooth, unfold};
use super::wake::{Key, Wake};
use crate::command::Command;
use crate::failure::Failure;
use crate::progress::Model;
use celis_nostr::Note;
use celis_trails::Trails;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

/// One drawn page: its lines, the line to keep in view, its key line, the id `c` copies, the Markdown
/// `m` copies, whether its lines hold a spinner that turns, what celis says above it in turn, and how long
/// until it is drawn again with no wake.
#[derive(Default)]
pub(crate) struct Page {
    pub lines: Vec<Styled>,
    pub picked: Option<usize>,
    pub keys: String,
    pub copy: Option<String>,
    pub markdown: Option<String>,
    pub spun: bool,
    pub remarks: Vec<Remark>,
    pub again: Option<Duration>,
}

/// A page a live screen draws from the model of a store and moves with keys.
pub(crate) trait Live {
    /// Forgets what it kept from the model before that the records `landed` change in `model`; all of
    /// it when `landed` is `None`.
    fn renew(&mut self, model: &Model, landed: Option<&[Note]>);

    /// The page it shows of `model`; `screen` follows its selected line when that moved.
    fn page(&mut self, model: &Model, screen: &mut Screen) -> Result<Page, Failure>;

    /// Moves as `key` asks on the page it last drew from `model`.
    fn press(&mut self, key: Key, model: &Model, screen: &mut Screen) -> Result<(), Failure>;

    /// Whether the person types into it, so `q` types and does not quit.
    fn typing(&self) -> bool {
        false
    }
}

/// The wakes of one run: whether records landed, whether the whole store must be read again, the
/// store read again, and the files of the records that landed.
#[derive(Default)]
struct Batch {
    landed: bool,
    linked: bool,
    store: Option<Box<Result<Trails, Failure>>>,
    paths: Vec<PathBuf>,
}

impl Command {
    /// Draws the page `start` makes from the store on the alternate screen, live; with no store, draws
    /// `empty` and writes nothing until one is made. Ends on an interrupt or a `q` it does not type.
    pub(crate) fn live<P: Live>(
        &self,
        out: &mut dyn Write,
        empty: Page,
        start: impl FnOnce(&Trails) -> Result<P, Failure>,
    ) -> Result<(), Failure> {
        self.screen(out, |out, wakes| {
            let mut screen = Screen {
                colour: self.colour(),
                ..Screen::default()
            };
            if !self.held()? {
                screen.draw(out, &[], &empty.lines, None, &empty.keys)?;
                while !self.held()? {
                    let wake = wakes.1.recv_timeout(Duration::from_millis(250));
                    if let Ok(Wake::Quit | Wake::Key(Key::Quit)) = wake {
                        return Ok(());
                    }
                }
            }
            let trails = self.trails()?;
            let mut page = start(&trails)?;
            self.follow(&mut page, trails, &mut screen, out, wakes)
        })
    }

    /// Draws `page` from the model of `trails`, the store, on `screen`, and again for each run of wakes
    /// from `events`; reads the files of records that land, and the whole store off this thread when it
    /// must.
    fn follow(
        &self,
        page: &mut impl Live,
        mut trails: Trails,
        screen: &mut Screen,
        out: &mut dyn Write,
        (send, events): (Sender<Wake>, Receiver<Wake>),
    ) -> Result<(), Failure> {
        let loads = send.clone();
        let _watcher = self.listen(send)?;
        let mut landed = None;
        std::thread::scope(|scope| loop {
            let batch = Model::of(&trails, |model| {
                page.renew(model, landed.as_deref());
                loop {
                    model.steps.at(crate::command::now());
                    let drawn = page.page(model, screen)?;
                    let wake = show(&drawn, screen, out, &events)?;
                    match drain(wake, &events, page, &drawn, model, screen)? {
                        Some(batch) if !batch.landed => {}
                        batch => return Ok::<_, Failure>(batch),
                    }
                }
            })?;
            let Some(batch) = batch else {
                return Ok(());
            };
            let read = batch.store.is_some();
            if let Some(store) = batch.store {
                trails = (*store)?;
            }
            if batch.linked {
                let loads = loads.clone();
                scope.spawn(move || loads.send(Wake::Store(Box::new(self.trails()))));
            }
            let taken = trails.take(&batch.paths);
            landed = (!read).then_some(taken);
        })
    }
}

/// Draws `page` on `screen`, under what celis says when it has a remark, the screen prints in colour and
/// leaves `TREE` rows under it, and under the trail scene too when the screen holds it whole: the scene
/// alone while it plays its opening when it held it from the start, then the card fading in and the rows
/// unfolding; else the scene still. Draws again each turn of its spinner or the dude, until a wake comes
/// or its time to draw again passes; that wake.
fn show(
    page: &Page,
    screen: &mut Screen,
    out: &mut dyn Write,
    events: &Receiver<Wake>,
) -> Result<Result<Wake, RecvTimeoutError>, Failure> {
    let due = page.again.map(|again| Instant::now() + again);
    let checking = page
        .lines
        .iter()
        .flat_map(|l| &l.spans)
        .any(|s| s.content.contains(DRAWN));
    if !checking {
        screen.checked = None;
    }
    let checked = *screen.checked.get_or_insert_with(Instant::now);
    let size = crate::command::size();
    let room = size.map(|s| usize::from(s.ws_row).saturating_sub(1));
    let width = size.map_or(0, |s| usize::from(s.ws_col).saturating_sub(1));
    let fits = |rows: usize| room.is_some_and(|r| r >= rows + TREE);
    let says = !page.remarks.is_empty() && screen.colour && fits(remark::ROWS) && width >= CARD;
    let wide = width >= 2 + scene::COLUMNS + 2;
    let scenery = says && wide && fits(scene::ROWS + remark::ROWS);
    let opened = *screen.opened.get_or_insert_with(Instant::now);
    let still = !*screen.played.get_or_insert(scenery);
    let turns = page.spun || says;
    let mut arriving;
    loop {
        let lines = match page.spun {
            true => spinner(&page.lines, spin(), drawn(checked)),
            false => page.lines.clone(),
        };
        let opening = opened.elapsed().as_secs_f64();
        let mut head = match (scenery, still) {
            (true, false) => fade(&scene(drawn(opened), width), opening, FADE),
            (true, true) => scene(PLAYS + drawn(opened), width),
            (false, _) => vec![],
        };
        let started = *screen.started.get_or_insert_with(crate::command::now);
        let fresh = news(&page.remarks, started);
        if let Some(remark) = turned(screen, &fresh).filter(|_| says) {
            if screen.remark.as_ref().map(|(shown, _)| shown) != Some(&remark) {
                let changed = screen.remark.is_some().then(Instant::now);
                screen.former = screen.remark.take().map(|(shown, _)| shown);
                screen.remark = Some((remark, changed));
            }
        }
        let (card, moving) = match says {
            true => said(screen, width),
            false => (vec![], false),
        };
        let since = match still {
            true => f64::MAX,
            false => opened.elapsed().as_secs_f64() - (PLAYS as u32 * TURN).as_secs_f64(),
        };
        let fading = opening < FADE;
        arriving =
            scenery && (fading || (0.0..length(card.len() + room.unwrap_or(0))).contains(&since));
        match (scenery, since) {
            (true, ..0.0) => screen.draw(out, &head, &[], None, "")?,
            (true, at) if at >= 0.0 && arriving => {
                head.extend(fade(&card, at, IN));
                screen.draw(out, &head, &unfold(&lines, at), page.picked, &page.keys)?;
            }
            _ => {
                head.extend(card);
                screen.draw(out, &head, &lines, page.picked, &page.keys)?;
            }
        }
        let left = due.map(|due| due.saturating_duration_since(Instant::now()));
        let turn = if arriving || moving { FRAME } else { TURN };
        let wait = match turns {
            true => left.map_or(turn, |left| left.min(turn)),
            false => left.unwrap_or(Duration::MAX),
        };
        match events.recv_timeout(wait) {
            Err(RecvTimeoutError::Timeout) if due.is_none_or(|due| Instant::now() < due) => {}
            wake => return Ok(wake),
        }
    }
}

/// What the card may show of `remarks`: the values that landed since `started`, else how the trail
/// stands.
fn news(remarks: &[Remark], started: u64) -> Vec<Remark> {
    let fresh: Vec<Remark> = remarks
        .iter()
        .filter(|r| r.at >= started)
        .cloned()
        .collect();
    match fresh.is_empty() {
        true => remarks.iter().filter(|r| r.at == 0).cloned().collect(),
        false => fresh,
    }
}

/// The remark the card shows now: the one it shows until it held it `HOLD`, or at once when that one
/// only says how the trail stands, then the oldest of `remarks` it has not shown yet; once none is
/// left, still the one it shows. It shows each once.
fn turned(screen: &mut Screen, remarks: &[Remark]) -> Option<Remark> {
    let standing = screen
        .remark
        .as_ref()
        .is_some_and(|(remark, _)| remark.at == 0);
    let held = standing || screen.since.is_none_or(|since| since.elapsed() >= HOLD);
    let next = remarks
        .iter()
        .filter(|r| !screen.seen.contains(r))
        .min_by_key(|r| r.at)
        .filter(|_| held)
        .cloned();
    if let Some(next) = next {
        screen.seen.push(next.clone());
        screen.since = Some(Instant::now());
        return Some(next);
    }
    let shown = screen.remark.as_ref().map(|(remark, _)| remark.clone());
    shown.or_else(|| remarks.last().cloned())
}

/// The seconds the card holds a remark at least before it shows the next new one.
const HOLD: Duration = Duration::from_secs(7);

/// The card of the remark `screen` shows, `width` wide, and whether it moves: once it changed, the
/// headline before fades out, then this one fades in while its words type in behind a caret that blinks a
/// while after them, then goes.
fn said(screen: &Screen, width: usize) -> (Vec<Styled>, bool) {
    let Some((remark, changed)) = &screen.remark else {
        return (vec![], false);
    };
    let Some(changed) = changed else {
        return (remark.rows(spin(), (None, false, 1.0), width), false);
    };
    let at = changed.elapsed().as_secs_f64();
    if let (Some(former), true) = (&screen.former, at < OUT) {
        let rows = former.rows(spin(), (None, false, 1.0 - smooth(at / OUT)), width);
        return (rows, true);
    }
    let at = at - OUT;
    let typed = (at * PACE) as usize;
    let caret = at < CARET && (typed < remark.length() || ((at * 2.5) as usize).is_multiple_of(2));
    let shown = smooth(at / IN);
    (
        remark.rows(spin(), (Some(typed), caret, shown), width),
        at < CARET,
    )
}

/// The seconds the headline before takes to fade out.
const OUT: f64 = 0.3;

/// The seconds a headline takes to fade in.
const IN: f64 = 0.45;

/// The characters a changed remark types each second.
const PACE: f64 = 45.0;

/// The seconds a changed remark shows its caret.
const CARET: f64 = 3.5;

/// The frames of the spinner since `at`.
fn drawn(at: Instant) -> usize {
    (at.elapsed().as_millis() / TURN.as_millis()) as usize
}

/// The time between two frames while rows arrive.
const FRAME: Duration = Duration::from_millis(16);

/// The fewest columns a screen needs to show celis's card.
const CARD: usize = 44;

/// The fewest rows a screen keeps for the tree under what celis says.
const TREE: usize = 8;

/// The run of wakes from `wake` on, with the wakes that wait in `events` up to a key that changes what
/// a page shows; records wait 50 ms for more. Each key copies from `drawn`, the page last drawn, or
/// moves `screen` and `page` on `model`. `None` on an interrupt.
fn drain(
    mut wake: Result<Wake, RecvTimeoutError>,
    events: &Receiver<Wake>,
    page: &mut impl Live,
    drawn: &Page,
    model: &Model,
    screen: &mut Screen,
) -> Result<Option<Batch>, Failure> {
    let mut batch = Batch::default();
    let mut laid = false;
    loop {
        match wake {
            Ok(Wake::Record(files)) => {
                batch.landed = true;
                batch.paths.extend(files);
            }
            Ok(Wake::Rescan) => (batch.landed, batch.linked) = (true, true),
            Ok(Wake::Store(read)) => (batch.landed, batch.store) = (true, Some(read)),
            Ok(Wake::Resize) => (screen.shown, screen.resized) = (vec![], true),
            Ok(Wake::Key(Key::Quit)) if !page.typing() => return Ok(None),
            Ok(Wake::Key(key)) => {
                laid = key.lays_out();
                screen.said = match (&key, &drawn.copy, &drawn.markdown) {
                    (Key::Copy, Some(id), _) => Some(clipboard(id, &format!("copied {id}"))),
                    (Key::Markdown, _, Some(text)) => Some(clipboard(text, "copied markdown")),
                    _ => None,
                };
                screen.press(&key);
                page.press(key, model, screen)?;
            }
            Ok(Wake::Quit) | Err(RecvTimeoutError::Disconnected) => return Ok(None),
            Err(RecvTimeoutError::Timeout) => return Ok(Some(batch)),
        }
        wake = match (batch.landed, laid) {
            (true, _) => events.recv_timeout(Duration::from_millis(50)),
            (false, false) => events.recv_timeout(Duration::ZERO),
            (false, true) => return Ok(Some(batch)),
        };
    }
}

#[cfg(test)]
mod tests;
