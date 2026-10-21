//! Where the person is in the graph view, and how each key moves her.

use crate::terminal::wake::Key;
use celis_trails::{Step, Steps};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// What the view shows: every trail of today, or one trail's graph.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Mode {
    All,
    Trail,
}

/// Where the person is: all trails or one, the trail shown and the trail picked among all, the card
/// selected and the row selected in it, the cards opened and the nestings unfolded, whether the rules
/// fill the screen with the rule selected and the first row shown, what she searches and the trail
/// picked before, whether the view follows the newest card and how far it is scrolled, whether it
/// scrolls to show the trail picked, and since when a new trail is above the screen. It keeps from the
/// last frame where each card and each trail stood, the steps the selected card lists, and the trails
/// and rules shown.
#[derive(Default)]
pub(super) struct View {
    pub mode: Option<Mode>,
    pub trail: Option<String>,
    pub pick: Option<String>,
    pub selected: Option<String>,
    pub row: Option<String>,
    pub opened: HashSet<String>,
    pub unfolded: HashSet<String>,
    pub rules: bool,
    pub rule: Option<String>,
    pub rule_top: usize,
    pub query: Option<String>,
    pub before: Option<String>,
    pub follow: bool,
    pub offset: f64,
    pub reveal: bool,
    pub above: Option<Instant>,
    pub cards: Vec<(String, u16, u16)>,
    pub listed: Vec<String>,
    pub trails: Vec<String>,
    pub matches: Vec<String>,
    pub learned: Vec<String>,
    pub places: HashMap<String, u16>,
    pub newest: Option<String>,
}

impl View {
    /// A view of `trails`: all of them, or the trail `named` when one is, the first picked, following
    /// the newest card.
    pub fn new(trails: &[Step], named: Option<&str>) -> View {
        let first = trails.first().map(|t| t.id().to_string());
        let mode = match named {
            Some(_) => Mode::Trail,
            None => Mode::All,
        };
        View {
            mode: Some(mode),
            trail: named.map(str::to_string).or(first.clone()),
            pick: first,
            follow: true,
            ..View::default()
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode.unwrap_or(Mode::All)
    }

    /// The key line: one for all trails, one for the rules, one for a search, and one for inside a
    /// trail at any depth, `esc` last and always back.
    pub fn keys(&self) -> &'static str {
        if self.query.is_some() {
            return "type to find · ↑↓ navigate · enter open · esc back";
        }
        if self.rules {
            return "↑↓ navigate · esc back";
        }
        match self.mode() {
            Mode::All => "↑↓ navigate · enter open · / find · r rules",
            Mode::Trail => {
                "←↑↓→ navigate · enter open · * open all · tab next trail · a all trails · r rules · esc back"
            }
        }
    }

    /// Shows the trail `id`'s graph, or all trails when it is none, the trail left picked.
    pub fn show(&mut self, id: Option<String>) {
        match id {
            None => {
                self.mode = Some(Mode::All);
                self.pick = self.trail.clone();
                self.reveal = true;
                self.places.clear();
            }
            Some(id) => {
                self.mode = Some(Mode::Trail);
                self.pick = Some(id.clone());
                self.trail = Some(id);
            }
        }
        self.selected = None;
        self.row = None;
        self.opened.clear();
        self.unfolded.clear();
        self.rules = false;
        self.offset = 0.0;
        self.follow = true;
    }

    /// Moves as `key` asks, the steps of the store in `steps`.
    pub fn press(&mut self, key: Key, steps: &Steps) {
        if self.query.is_some() {
            return self.search(key);
        }
        if let Key::Char('/') = key {
            if self.mode() == Mode::Trail {
                self.show(None);
            }
            self.before = self.pick.clone();
            self.query = Some(String::new());
            return;
        }
        if self.rules {
            return self.read(key);
        }
        if let Key::Char('r') = key {
            self.rules = true;
            self.rule = self.learned.first().cloned();
            self.rule_top = 0;
            return;
        }
        match self.mode() {
            Mode::All => self.all(key),
            Mode::Trail => self.inside(key, steps),
        }
    }

    /// Takes a key on all trails: ↑↓ pick a trail, enter opens its graph.
    fn all(&mut self, key: Key) {
        match key {
            Key::Up => self.pick_by(-1),
            Key::Down => self.pick_by(1),
            Key::Enter if self.pick.is_some() => self.show(self.pick.clone()),
            _ => {}
        }
    }

    /// Picks the trail `by` places from the one picked among those shown, stopping at the ends.
    fn pick_by(&mut self, by: isize) {
        let trails = &self.matches;
        if trails.is_empty() {
            return;
        }
        let at = trails
            .iter()
            .position(|t| Some(t) == self.pick.as_ref())
            .unwrap_or(0);
        let to = (at as isize + by).clamp(0, trails.len() as isize - 1) as usize;
        self.pick = Some(trails[to].clone());
        self.reveal = true;
    }

    /// Takes a key inside a trail: the arrows move between cards and through an opened card's rows,
    /// enter opens or closes a card or a nesting, `*` opens everything under the selection, tab goes to
    /// the next trail, `a` to all trails, `f` follows the newest card, and esc steps back one level.
    fn inside(&mut self, key: Key, steps: &Steps) {
        match key {
            Key::Up | Key::Down | Key::Left | Key::Right => {
                let opened = self
                    .selected
                    .as_ref()
                    .is_some_and(|s| self.opened.contains(s));
                if opened && self.walk(&key, steps) {
                    self.follow = false;
                    return;
                }
                self.step(&key);
            }
            Key::Enter => match (&self.row, &self.selected) {
                (Some(row), _) => {
                    let nested = steps.get(row).is_some_and(|s| !s.children().is_empty());
                    if nested && !self.unfolded.remove(row) {
                        self.unfolded.insert(row.clone());
                    }
                }
                (None, Some(selected)) => {
                    if !self.opened.remove(selected) {
                        self.opened.insert(selected.clone());
                    }
                }
                (None, None) => {}
            },
            Key::Star => {
                let Some(selected) = self.selected.clone() else {
                    return;
                };
                self.opened.insert(selected.clone());
                let root = self.row.clone().unwrap_or(selected);
                if let Some(root) = steps.get(&root) {
                    let nested = root.tree().into_iter().filter(|s| !s.children().is_empty());
                    self.unfolded.extend(nested.map(|s| s.id().to_string()));
                }
            }
            Key::Tab | Key::BackTab => {
                let at = self
                    .trails
                    .iter()
                    .position(|t| Some(t) == self.trail.as_ref());
                let by = if matches!(key, Key::Tab) {
                    1
                } else {
                    self.trails.len() - 1
                };
                if let Some(at) = at {
                    let to = self.trails[(at + by) % self.trails.len()].clone();
                    self.show(Some(to));
                }
            }
            Key::All => self.show(None),
            Key::Full => (self.selected, self.row, self.follow) = (None, None, true),
            Key::Escape => self.back(),
            _ => {}
        }
    }

    /// Steps back one level: from a row to its card, from an opened card to the card closed, from a
    /// card to none, and from none to all trails.
    fn back(&mut self) {
        if self.row.take().is_some() {
            return;
        }
        if let Some(selected) = &self.selected {
            if !self.opened.remove(selected) {
                self.selected = None;
                self.follow = true;
            }
            return;
        }
        self.show(None);
    }

    /// Moves the selection by an arrow `key` among the cards where the last frame drew them: along the
    /// chain, or a row of cards up or down; the newest card when none was selected.
    fn step(&mut self, key: &Key) {
        if self.cards.is_empty() {
            return;
        }
        self.follow = false;
        let Some(at) = self
            .cards
            .iter()
            .position(|c| Some(&c.0) == self.selected.as_ref())
        else {
            self.selected = self.newest.clone().or(Some(self.cards[0].0.clone()));
            return;
        };
        let (_, x, y) = self.cards[at];
        let to = match key {
            Key::Right => (at + 1).min(self.cards.len() - 1),
            Key::Left => at.saturating_sub(1),
            _ => {
                let mut rows: Vec<u16> = self.cards.iter().map(|c| c.2).collect();
                rows.dedup();
                let r = rows.iter().position(|r| *r == y).unwrap_or(0) as isize;
                let r = r + if matches!(key, Key::Down) { 1 } else { -1 };
                let Some(row) = usize::try_from(r).ok().and_then(|r| rows.get(r)) else {
                    return;
                };
                let same = self.cards.iter().enumerate().filter(|(_, c)| c.2 == *row);
                same.min_by_key(|(_, c)| c.1.abs_diff(x))
                    .map_or(at, |(n, _)| n)
            }
        };
        self.selected = Some(self.cards[to].0.clone());
    }

    /// Moves through the rows of the opened card: up and down a row, → unfolds a row or steps into it,
    /// ← folds it or steps out to its parent; out of the card past its ends. Whether it moved.
    fn walk(&mut self, key: &Key, steps: &Steps) -> bool {
        let rows = &self.listed;
        let Some(row) = self.row.clone() else {
            if matches!(key, Key::Down) && !rows.is_empty() {
                self.row = Some(rows[0].clone());
                return true;
            }
            return false;
        };
        let at = rows.iter().position(|r| *r == row).unwrap_or(0);
        let step = steps.get(&row);
        let nested = step.is_some_and(|s| !s.children().is_empty());
        match key {
            Key::Down if at + 1 < rows.len() => self.row = Some(rows[at + 1].clone()),
            Key::Down => {
                self.row = None;
                return false;
            }
            Key::Up => self.row = (at > 0).then(|| rows[at - 1].clone()),
            Key::Right if nested && !self.unfolded.contains(&row) => {
                self.unfolded.insert(row);
            }
            Key::Right if nested => {
                let first = step.and_then(|s| s.children().first().map(|k| k.id().to_string()));
                self.row = first.or(Some(row));
            }
            Key::Left if self.unfolded.remove(&row) => {}
            Key::Left => {
                let parent = step.and_then(|s| s.under()).map(|p| p.id().to_string());
                self.row = parent.filter(|p| Some(p) != self.selected.as_ref());
            }
            _ => {}
        }
        true
    }

    /// Takes a key while the rules fill the screen: ↑↓ move through them, esc or `r` close them.
    fn read(&mut self, key: Key) {
        let at = self
            .learned
            .iter()
            .position(|r| Some(r) == self.rule.as_ref())
            .unwrap_or(0);
        match key {
            Key::Up => self.rule = self.learned.get(at.saturating_sub(1)).cloned(),
            Key::Down => {
                let to = (at + 1).min(self.learned.len().saturating_sub(1));
                self.rule = self.learned.get(to).cloned();
            }
            Key::Escape | Key::Char('r') => self.rules = false,
            _ => {}
        }
    }

    /// Takes a key while the person searches all trails: a sign adds to the words, erase takes one
    /// off, ↑↓ pick among the matches, enter opens the one picked, esc clears the words.
    fn search(&mut self, key: Key) {
        let Some(query) = self.query.as_mut() else {
            return;
        };
        match key {
            Key::Escape => {
                self.query = None;
                self.pick = self.before.take().or(self.pick.take());
                self.reveal = true;
            }
            Key::Enter => {
                self.query = None;
                if self.pick.is_some() {
                    self.show(self.pick.clone());
                }
            }
            Key::Up => self.pick_by(-1),
            Key::Down => self.pick_by(1),
            Key::Erase => {
                query.pop();
                self.offset = 0.0;
                self.places.clear();
            }
            key => {
                if let Some(c) = key.typed() {
                    query.push(c);
                    self.offset = 0.0;
                    self.places.clear();
                }
            }
        }
    }

    /// Whether `trail`, with its `project`, holds every word the person searches.
    pub fn finds(&self, title: &str, project: &str) -> bool {
        let Some(query) = &self.query else {
            return true;
        };
        let text = format!("{title} {project}").to_lowercase();
        query
            .to_lowercase()
            .split_whitespace()
            .all(|w| text.contains(w))
    }
}

#[cfg(test)]
mod tests;
