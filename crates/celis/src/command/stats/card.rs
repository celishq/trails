//! One card of stats: the face it shows among the cards, and the table it opens to.

use super::table::Table;
use crate::progress::window::DAYS;
use crate::progress::{Model, Window};
use crate::terminal::canvas::Styled;
use crate::terminal::ink::Ink;
use effect::effect;
use outcome::outcome;
use projects::projects;
use reuse::reuse;
use rules::rules;
use time::time;
use work::work;

/// A card of stats: its title, the rows of its face, and the table it opens to.
pub(crate) struct Card {
    pub title: String,
    pub face: Vec<Styled>,
    pub table: Table,
}

/// The count of cards: effect, reuse, rules used, work, time, outcome and projects.
pub(crate) const CARDS: usize = 7;

/// The cards of `model` at its time: the effect `full` characters wide inside, the others `inside`;
/// their numbers plain text when `plain`.
pub(crate) fn cards(model: &Model, full: usize, inside: usize, plain: bool) -> [Card; CARDS] {
    let window = Window::new(model.steps);
    let ink = Ink { plain };
    [
        effect(&window, full, ink),
        reuse(model, &window, inside, ink),
        rules(model, &window, inside, ink),
        work(&window, inside, ink),
        time(&window.last, inside, ink),
        outcome(&window.last, inside, ink),
        projects(&window.last, inside, ink),
    ]
}

/// `no <what> in 14 days`, the line of a card with nothing in its period.
fn nothing(what: &str) -> String {
    format!("no {what} in {DAYS} days")
}

mod change;
mod daily;
mod effect;
mod face;
mod outcome;
mod projects;
mod reuse;
mod rules;
#[cfg(test)]
mod tests;
mod time;
mod work;
