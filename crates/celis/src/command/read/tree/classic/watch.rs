//! `celis tree` on a terminal: the tree, drawn again each time the store or a linked folder changes.

use crate::command::read::tree::classic::{Frame, Item, Layout, View, EMPTY, MORE};
use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::progress::Model;
use crate::terminal::canvas::Styled;
use crate::terminal::live::{Live, Page};
use crate::terminal::screen::{until, Screen};
use crate::terminal::wake::Key;
use celis_nostr::Note;
use celis_trails::TrailStore;
use std::io::Write;

impl Command {
    /// Draws the tree `args` names on the alternate screen, then again when a record lands in the store
    /// or a linked folder, a time it shows changes, the spinner turns, a key is read, or the terminal's
    /// size changes, and sleeps between. With no store, it writes nothing and waits for one. On an
    /// interrupt or `q`, leaves the alternate screen with nothing printed, and ends.
    pub(crate) fn classic_watch(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        if let (false, Some(id)) = (self.held()?, args.words.get(1)) {
            return Err(Failure::no_step(id));
        }
        let empty = vec![Styled::from(EMPTY)];
        let waiting = Page {
            lines: empty.clone(),
            keys: legend(None),
            ..Page::default()
        };
        self.live(out, waiting, |trails| {
            if let Some(id) = args.words.get(1) {
                trails.steps().find(id)?;
            }
            Ok(Watch {
                command: self,
                args,
                named: args.words.len() > 1,
                empty,
                view: View {
                    watch: true,
                    spin: true,
                    ..View::of(args)
                },
                layout: Layout::default(),
                frame: None,
            })
        })
    }
}

/// A watch of the tree `args` names: whether it names an id, what it prints with no trail, how it shows
/// the tree, the day groups it laid out, and the frame it last drew.
struct Watch<'a> {
    command: &'a Command,
    args: &'a Args,
    named: bool,
    empty: Vec<Styled>,
    view: View,
    layout: Layout,
    frame: Option<Frame>,
}

impl Live for Watch<'_> {
    fn renew(&mut self, model: &Model, landed: Option<&[Note]>) {
        self.layout.renew(model, landed);
    }

    /// The tree of `model`, its spinner turning, drawn again when a time it shows changes; the
    /// selection moves to the nearest line still shown when its line left.
    fn page(&mut self, model: &Model, screen: &mut Screen) -> Result<Page, Failure> {
        let width = self.command.columns().map(|c| c.saturating_sub(1));
        let mut frame =
            self.command
                .classic_frame(model, Some(&self.layout), self.args, width, &self.view)?;
        if let Some(id) = self.view.selected.clone() {
            if !frame.order.iter().any(|i| i.id == id) {
                let known = self.frame.as_ref().map_or(&[][..], |f| &f.order);
                self.view.selected = kept(known, &frame.order, &id);
                screen.follow = true;
                return self.page(model, screen);
            }
        }
        let lines = match frame.text.is_empty() && !self.named {
            true => self.empty.clone(),
            false => std::mem::take(&mut frame.text),
        };
        let selected = frame
            .order
            .iter()
            .find(|i| self.view.selected.as_ref() == Some(&i.id));
        let copy = selected.and_then(Item::copy);
        let part = |item: &Item| {
            crate::command::read::report::part(model.steps, model.patterns, model.notes, &item.id)
        };
        let markdown = selected
            .filter(|_| copy.is_some())
            .and_then(|item| part(item).ok());
        let page = Page {
            lines,
            picked: frame.picked,
            keys: legend(copy.as_deref()),
            copy,
            markdown,
            spun: frame.spun,
            scene: false,
            again: frame.next.map(until),
        };
        self.frame = Some(frame);
        Ok(page)
    }

    /// Moves the selection, or opens or closes lines.
    fn press(&mut self, key: Key, _: &Model, screen: &mut Screen) -> Result<(), Failure> {
        let order = self.frame.as_ref().map_or(&[][..], |f| &f.order);
        let before = self.view.selected.clone();
        press(key, &mut self.view, order, self.named);
        screen.follow |= self.view.selected != before;
        Ok(())
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
/// pattern or step, or everything under it too, selects the one it is under, or turns full on or off, and
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
        (
            Key::PageUp
            | Key::PageDown
            | Key::Space
            | Key::Tab
            | Key::BackTab
            | Key::Back
            | Key::Forth
            | Key::Escape
            | Key::Top
            | Key::End
            | Key::Copy
            | Key::Markdown
            | Key::Char(_)
            | Key::Erase
            | Key::Quit,
            _,
        ) => {}
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
