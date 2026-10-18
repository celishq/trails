//! `celis tree` on a terminal: the tree, drawn again each time the store or a linked folder changes.

use super::screen::{clipboard, spin, until, Screen, TURN};
use super::wake::{Key, Wake};
use crate::command::tree::{Item, Model, View, EMPTY, MORE};
use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::progress::Reuse;
use std::io::Write;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

impl Command {
    /// Draws the tree `args` names on the alternate screen, then again when a record lands in the store
    /// or a linked folder, a time it shows changes, the spinner turns, a key is read, or the terminal's
    /// size changes, and sleeps between. With no store, it writes nothing and waits for one. On an
    /// interrupt or `q`, leaves the alternate screen with nothing printed, and ends.
    pub(crate) fn watch(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        if let (false, Some(id)) = (self.held()?, args.words.get(1)) {
            return Err(Failure::no_step(id));
        }
        self.screen(out, |out, wakes| self.watched(args, out, wakes))
    }

    /// The watch's loop: draws each tree it reads from the store, until an interrupt.
    fn watched(
        &self,
        args: &Args,
        out: &mut dyn Write,
        (send, events): (Sender<Wake>, Receiver<Wake>),
    ) -> Result<(), Failure> {
        let mut screen = Screen::default();
        let empty = format!("{EMPTY}\n");
        if !self.waited(&mut screen, out, (&empty, &legend(None)), &events)? {
            return Ok(());
        }
        let mut trails = self.trails()?;
        if let Some(id) = args.words.get(1) {
            trails.steps().find(id)?;
        }
        let _watcher = self.listen(send)?;
        let named = args.words.len() > 1;
        let mut view = View {
            watch: true,
            ..View::of(args)
        };
        let mut known: Vec<Item> = vec![];
        loop {
            let steps = trails.steps();
            let decisions = trails.decisions();
            let reuse = Reuse::new(&steps);
            let model = Model {
                steps: &steps,
                decisions: &decisions,
                reuse: &reuse,
            };
            let linked = loop {
                steps.at(crate::command::now());
                view.spin = Some(spin());
                let width = self.columns().map(|c| c.saturating_sub(1));
                let frame = self.frame(&model, args, width, &view)?;
                if let Some(id) = view.selected.clone() {
                    if !frame.order.iter().any(|i| i.id == id) {
                        view.selected = kept(&known, &frame.order, &id);
                        screen.follow = true;
                        continue;
                    }
                }
                let tree = match frame.text.is_empty() && !named {
                    true => empty.clone(),
                    false => frame.text,
                };
                let selected = frame
                    .order
                    .iter()
                    .find(|i| view.selected.as_ref() == Some(&i.id));
                let copy = selected.and_then(Item::copy);
                screen.draw(out, &tree, frame.picked, &legend(copy.as_deref()))?;
                let wait = frame.next.map(until);
                let wait = match frame.spun {
                    true => wait.map_or(TURN, |w| w.min(TURN)),
                    false => wait.unwrap_or(Duration::MAX),
                };
                let mut wake = events.recv_timeout(wait);
                let (mut landed, mut linked, mut laid) = (false, false, false);
                loop {
                    match wake {
                        Ok(Wake::Record(from)) => (landed, linked) = (true, linked || from),
                        Ok(Wake::Resize) => screen.shown.clear(),
                        Ok(Wake::Key(key)) => {
                            let selected = frame
                                .order
                                .iter()
                                .find(|i| view.selected.as_ref() == Some(&i.id));
                            let copy = selected.and_then(Item::copy);
                            let chosen = selected.filter(|_| copy.is_some()).map(|i| i.id.clone());
                            laid = key.lays_out();
                            screen.said = match (&key, &copy, &chosen) {
                                (Key::Copy, Some(id), _) => {
                                    Some(clipboard(id, &format!("copied {id}")))
                                }
                                (Key::Markdown, _, Some(id)) => {
                                    let markdown =
                                        crate::command::report::part(&steps, &decisions, id)?;
                                    Some(clipboard(&markdown, "copied markdown"))
                                }
                                _ => None,
                            };
                            let selected = view.selected.clone();
                            screen.press(&key);
                            press(key, &mut view, &frame.order, named);
                            screen.follow |= view.selected != selected;
                        }
                        Ok(Wake::Quit) | Err(RecvTimeoutError::Disconnected) => return Ok(()),
                        Err(RecvTimeoutError::Timeout) => break,
                    }
                    wake = match (landed, laid) {
                        (true, _) => events.recv_timeout(Duration::from_millis(50)),
                        (false, false) => events.recv_timeout(Duration::ZERO),
                        (false, true) => break,
                    };
                }
                known = frame.order;
                if landed {
                    break linked;
                }
            };
            match linked {
                true => trails = self.trails()?,
                false => _ = trails.fresh(),
            }
        }
    }
}

/// The line on the last row of a watch with the keys it reads, `c` with the id `copy` it copies.
fn legend(copy: Option<&str>) -> String {
    let copy = copy.map_or(String::new(), |id| {
        format!(" · c copy {id} · m copy markdown")
    });
    format!("↑↓ select · ←→ close open · * open all under{copy} · PgUp PgDn scroll · f full · a all · q quit")
}

/// Changes `view` as `key` asks: moves the selection along `order`, opens or closes the selected trail,
/// rule or step, or everything under it too, selects the one it is under, or turns full on or off, and
/// all when the watch `named` no id.
fn press(key: Key, view: &mut View, order: &[Item], named: bool) {
    let at = view
        .selected
        .as_ref()
        .and_then(|s| order.iter().position(|o| &o.id == s));
    let last = order.len().saturating_sub(1);
    match (key, at.map(|at| &order[at])) {
        (Key::Down, None) => view.selected = order.first().map(|o| o.id.clone()),
        (Key::Down, Some(_)) => view.selected = at.map(|at| order[(at + 1).min(last)].id.clone()),
        (Key::Up, Some(_)) => view.selected = at.map(|at| order[at.saturating_sub(1)].id.clone()),
        (Key::Enter, Some(item)) => turn(view, &item.id, item.open, !item.open),
        (Key::Right, Some(item)) => turn(view, &item.id, item.open, true),
        (Key::Left, Some(item)) if item.open => turn(view, &item.id, true, false),
        (Key::Left, Some(item)) => {
            if let Some(parent) = &item.parent {
                view.selected = Some(parent.clone());
            }
        }
        (Key::Star, Some(item)) => {
            let opened = |view: &View, id: &String| view.full != view.flipped.contains(id);
            let open = !(item.open && item.under.iter().all(|id| opened(view, id)));
            turn(view, &item.id, item.open, open);
            for id in &item.under {
                let now = opened(view, id);
                turn(view, id, now, open);
            }
        }
        (Key::Up | Key::Enter | Key::Right | Key::Left | Key::Star, None) => {}
        (Key::PageUp | Key::PageDown | Key::Top | Key::End | Key::Copy | Key::Markdown, _) => {}
        (Key::Full, _) => view.full = !view.full,
        (Key::All, _) => view.all = !view.all && !named,
    }
}

/// The line to select once the selected `id` left `order`: the nearest line it was under in `known`
/// that is still in `order`, else the first.
fn kept(known: &[Item], order: &[Item], id: &str) -> Option<String> {
    let mut at = Some(id.to_string());
    while let Some(id) = at {
        if order.iter().any(|i| i.id == id) {
            return Some(id);
        }
        at = known
            .iter()
            .find(|i| i.id == id)
            .and_then(|i| i.parent.clone());
    }
    order.first().map(|i| i.id.clone())
}

/// Opens the line `id` in `view` when `open`, else closes it; it is open when `now`. `Show more` opens
/// as every trail by day.
fn turn(view: &mut View, id: &str, now: bool, open: bool) {
    if id == MORE {
        view.all = open;
        return;
    }
    if now != open && !view.flipped.remove(id) {
        view.flipped.insert(id.to_string());
    }
}
