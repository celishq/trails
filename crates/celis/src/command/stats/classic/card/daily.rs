//! A count of each day of a window, as the reuse, patterns and work cards show it.

use super::super::{Period, Window};

/// A count of each day of a window: the start of each day, oldest first, its count, and the count of
/// the 14 days before.
pub(super) struct Daily {
    pub starts: Vec<u64>,
    pub counts: Vec<usize>,
    pub before: usize,
}

impl Daily {
    /// What `count` counts in each day of `window`, and in the 14 days before.
    pub fn new<'a>(window: &Window<'a>, count: impl Fn(&Period<'a>) -> usize) -> Daily {
        Daily {
            starts: window.days.iter().map(Period::from).collect(),
            counts: window.days.iter().map(&count).collect(),
            before: count(&window.before),
        }
    }
}
