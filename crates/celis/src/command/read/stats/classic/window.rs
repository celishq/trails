//! The spans of time the cards of stats compare.

use super::Period;
use crate::text::clock::midnight;
use celis_trails::Steps;

/// The days of one span the cards compare.
pub const DAYS: u64 = 14;

/// The seconds of one day.
pub const DAY: u64 = 86_400;

/// The spans of a store the cards compare at its time: the last 14 days, the 14 days before them, and
/// each of the last 14 days from its midnight, oldest first.
pub struct Window<'a> {
    pub last: Period<'a>,
    pub before: Period<'a>,
    pub days: Vec<Period<'a>>,
}

impl<'a> Window<'a> {
    pub fn new(steps: &'a Steps<'a>) -> Window<'a> {
        let now = steps.now();
        let span = DAYS * DAY;
        let today = midnight(now);
        let days = (0..DAYS).rev().map(|i| today.saturating_sub(i * DAY));
        Window {
            last: Period::new(steps, now.saturating_sub(span), now + 1),
            before: Period::new(
                steps,
                now.saturating_sub(2 * span),
                now.saturating_sub(span),
            ),
            days: days.map(|d| Period::new(steps, d, d + DAY)).collect(),
        }
    }
}

#[cfg(test)]
mod tests;
