//! One card of stats: the face it shows among the cards, and the table it opens to.

use super::table::Table;
use crate::progress::{Model, Period};
use crate::terminal::canvas::Styled;
use crate::terminal::ink::Ink;
use effect::effect;
use outcome::outcome;
use reuse::{followed, rules};
use work::{projects, time, work};

/// A card of stats: its title, the rows of its face, and the table it opens to.
pub(crate) struct Sheet {
    pub title: String,
    pub face: Vec<Styled>,
    pub table: Table,
}

/// The days of one period a card compares.
const DAYS: u64 = 14;

/// The seconds of one period a card compares.
pub(crate) const SPAN: u64 = DAYS * DAY;

const DAY: u64 = 86_400;

/// The count of cards: effect, reuse, rules used, work, time, outcome and projects.
pub(crate) const CARDS: usize = 7;

/// The cards of `model` at its time: the effect `full` characters wide inside, the others `inside`;
/// their numbers plain text when `plain`.
pub(crate) fn sheets(model: &Model, full: usize, inside: usize, plain: bool) -> [Sheet; CARDS] {
    let (steps, reuse) = (model.steps, model.reuse);
    let now = steps.now();
    let last = Period::new(steps, now.saturating_sub(SPAN), now + 1);
    let before = Period::new(
        steps,
        now.saturating_sub(2 * SPAN),
        now.saturating_sub(SPAN),
    );
    let ink = Ink { plain };
    [
        effect(&last, &before, full, ink),
        followed(reuse, inside, ink),
        rules(model.rules, reuse, inside, ink),
        work(steps, &before, now, inside, ink),
        time(&last, inside, ink),
        outcome(&last, inside, ink),
        projects(&last, inside, ink),
    ]
}

/// `no step <what> in 14 days`, the line of a card with nothing in its period.
fn nothing(what: &str) -> String {
    format!("no step {what} in {DAYS} days")
}

mod effect;
mod outcome;
mod reuse;
#[cfg(test)]
mod tests;
mod work;
