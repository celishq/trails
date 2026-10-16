//! The kind of a line of the view.

/// What a line of the view holds.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Trail,
    Rule,
    Step,
    Proof,
    Changed,
    Dropped,
}

impl Kind {
    /// Every kind, in the order a count of them prints.
    pub const ALL: [Kind; 6] = [
        Kind::Trail,
        Kind::Rule,
        Kind::Step,
        Kind::Proof,
        Kind::Changed,
        Kind::Dropped,
    ];

    /// The word a line gives its kind.
    pub fn text(self) -> &'static str {
        match self {
            Kind::Trail => "trail",
            Kind::Rule => "rule",
            Kind::Step => "step",
            Kind::Proof => "proof",
            Kind::Changed => "changed",
            Kind::Dropped => "dropped",
        }
    }

    /// `<n> <kind>s` for a kind that is a noun, else `<n> <kind>`.
    pub fn count(self, n: usize) -> String {
        match self {
            Kind::Changed | Kind::Dropped => format!("{n} {}", self.text()),
            noun => crate::command::line::plural(n, noun.text()),
        }
    }
}
