//! A page drawn live from a store: again each time a record lands, a key is read, or its time to draw
//! again comes.

use super::canvas::Styled;
use super::look::{spin, spinner, TURN};
use super::screen::{clipboard, Screen};
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
/// `m` copies, whether its lines hold a spinner that turns, and how long until it is drawn again with
/// no wake.
#[derive(Default)]
pub(crate) struct Page {
    pub lines: Vec<Styled>,
    pub picked: Option<usize>,
    pub keys: String,
    pub copy: Option<String>,
    pub markdown: Option<String>,
    pub spun: bool,
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
    /// `empty` and writes nothing until one is made. Ends on an interrupt or `q`.
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
                screen.draw(out, &empty.lines, None, &empty.keys)?;
                while !self.held()? {
                    if let Ok(Wake::Quit) = wakes.1.recv_timeout(Duration::from_millis(250)) {
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

/// Draws `page` on `screen`, again each turn of its spinner, until a wake comes or its time to draw
/// again passes; that wake.
fn show(
    page: &Page,
    screen: &mut Screen,
    out: &mut dyn Write,
    events: &Receiver<Wake>,
) -> Result<Result<Wake, RecvTimeoutError>, Failure> {
    let due = page.again.map(|again| Instant::now() + again);
    loop {
        let lines = match page.spun {
            true => spinner(&page.lines, spin()),
            false => page.lines.clone(),
        };
        screen.draw(out, &lines, page.picked, &page.keys)?;
        let left = due.map(|due| due.saturating_duration_since(Instant::now()));
        let wait = match page.spun {
            true => left.map_or(TURN, |left| left.min(TURN)),
            false => left.unwrap_or(Duration::MAX),
        };
        match events.recv_timeout(wait) {
            Err(RecvTimeoutError::Timeout) if due.is_none_or(|due| Instant::now() < due) => {}
            wake => return Ok(wake),
        }
    }
}

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
            Ok(Wake::Resize) => screen.shown.clear(),
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
