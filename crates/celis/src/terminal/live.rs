//! A page drawn live from a store: again each time a record lands, a key is read, or its wait ends.

use super::screen::Screen;
use super::wake::{Key, Wake};
use crate::command::Command;
use crate::failure::Failure;
use crate::progress::Model;
use celis_trails::Trails;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

/// A page a live screen draws from the model of a store and moves with keys.
pub(crate) trait Live {
    /// Forgets what it kept from the model before.
    fn renew(&mut self);

    /// Draws itself from `model` on `screen` until a wake comes or its wait ends; that wake.
    fn wait(
        &mut self,
        model: &Model,
        screen: &mut Screen,
        out: &mut dyn Write,
        events: &Receiver<Wake>,
    ) -> Result<Result<Wake, RecvTimeoutError>, Failure>;

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
    /// Draws `page` from the model of `trails`, the store, on `screen`, and again for each run of wakes
    /// from `events`; reads the files of records that land, and the whole store off this thread when it
    /// must. Ends on an interrupt or `q`.
    pub(crate) fn live(
        &self,
        page: &mut impl Live,
        mut trails: Trails,
        screen: &mut Screen,
        out: &mut dyn Write,
        (send, events): (Sender<Wake>, Receiver<Wake>),
    ) -> Result<(), Failure> {
        let loads = send.clone();
        let _watcher = self.listen(send)?;
        std::thread::scope(|scope| loop {
            let batch = Model::of(&trails, |model| {
                page.renew();
                loop {
                    model.steps.at(crate::command::now());
                    let wake = page.wait(model, screen, out, &events)?;
                    match drain(wake, &events, page, model, screen)? {
                        Some(batch) if !batch.landed => {}
                        batch => return Ok::<_, Failure>(batch),
                    }
                }
            })?;
            let Some(batch) = batch else {
                return Ok(());
            };
            if let Some(read) = batch.store {
                trails = (*read)?;
            }
            if batch.linked {
                let loads = loads.clone();
                scope.spawn(move || loads.send(Wake::Store(Box::new(self.trails()))));
            }
            trails.take(&batch.paths);
        })
    }
}

/// The run of wakes from `wake` on, with the wakes that wait in `events` up to a key that opens or
/// closes lines; records wait 50 ms for more. Each key moves `screen` and `page` on `model`. `None` on
/// an interrupt.
fn drain(
    mut wake: Result<Wake, RecvTimeoutError>,
    events: &Receiver<Wake>,
    page: &mut impl Live,
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
