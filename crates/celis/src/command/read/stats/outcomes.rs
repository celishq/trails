//! How the steps of a span of days ended, and the shares stats reads from them.

/// How the steps that closed in a span ended, each step once: done following past work, done new, done
/// and revised later, dropped, or expired; and how many of the done ones applied a pattern.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub(super) struct Outcomes {
    pub reused: usize,
    pub new: usize,
    pub reworked: usize,
    pub dropped: usize,
    pub expired: usize,
    pub applied: usize,
}

impl Outcomes {
    /// The outcomes of every span of `spans` together.
    pub fn sum(spans: &[Outcomes]) -> Outcomes {
        spans.iter().fold(Outcomes::default(), |a, s| Outcomes {
            reused: a.reused + s.reused,
            new: a.new + s.new,
            reworked: a.reworked + s.reworked,
            dropped: a.dropped + s.dropped,
            expired: a.expired + s.expired,
            applied: a.applied + s.applied,
        })
    }

    /// The steps done: reused, new and reworked.
    pub fn successful(self) -> usize {
        self.reused + self.new + self.reworked
    }

    /// The steps that ended: successful, dropped and expired.
    pub fn resolved(self) -> usize {
        self.successful() + self.dropped + self.expired
    }

    /// The steps done that no later step revised.
    pub fn first_pass(self) -> usize {
        self.reused + self.new
    }

    /// The reused steps among the successful ones, in whole per cent.
    pub fn reuse_rate(self) -> i64 {
        percent(self.reused, self.successful())
    }

    /// The first-pass steps among the resolved ones, in whole per cent.
    pub fn first_pass_rate(self) -> i64 {
        percent(self.first_pass(), self.resolved())
    }
}

/// `part` of `whole` in whole per cent; none of nothing.
pub(super) fn percent(part: usize, whole: usize) -> i64 {
    match whole {
        0 => 0,
        _ => (part as f64 / whole as f64 * 100.0).round_ties_even() as i64,
    }
}
