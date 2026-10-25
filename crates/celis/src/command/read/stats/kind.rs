//! What a measure of stats counts, which says how its value and its change read.

use super::sheet::Group;
use crate::terminal::canvas::{paint, BOLD, DIM, GREEN, RED};
use crate::text::clock::took;

/// What a measure counts: a share in whole per cent, a count, or a time in seconds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Kind {
    Share,
    Count,
    Time,
}

impl Kind {
    /// `value` as it reads: `57%`, `44`, or `3m 49s`.
    pub fn text(self, value: i64) -> String {
        match self {
            Kind::Share => format!("{value}%"),
            Kind::Count => value.to_string(),
            Kind::Time => took(value.max(0) as u64),
        }
    }

    /// The change from `prior` to `now`: its arrow and size, `pp` for a share, green when better and rose
    /// when worse, `= no change` when none; the prior value itself left out. `no prior steps` with no
    /// value before, nothing with no value now.
    pub fn change(self, now: Option<i64>, prior: Option<i64>, up: bool) -> Vec<Group> {
        let Some(prior) = prior else {
            return vec![vec![paint(DIM, " no prior steps")]];
        };
        let Some(now) = now else {
            return vec![];
        };
        let moved = now - prior;
        let head = match moved {
            0 => paint(DIM, " = no change"),
            _ => {
                let size = match self {
                    Kind::Share => format!("{} pp", moved.abs()),
                    _ => self.text(moved.abs()),
                };
                let arrow = if moved > 0 { "▲" } else { "▼" };
                let look = if (moved > 0) == up { GREEN } else { RED };
                paint(look.patch(BOLD), &format!(" {arrow} {size}"))
            }
        };
        vec![vec![head]]
    }
}
