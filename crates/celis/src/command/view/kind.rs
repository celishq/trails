//! The kind of a line of the view.

/// What a line of the view holds.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Trail,
    Rule,
    Step,
    Proof,
    Changed,
    Dropped,
}

impl Kind {
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
}
