//! Who stands behind the proof of a done step.

use celis_trails::{By, Done, Step};

/// Who stands behind the proof of a done step: the check that celis ran, a judge that did not do the work,
/// the person, or only the agent's word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proof {
    Checked,
    Judged,
    Person,
    Claim,
}

impl Proof {
    /// Each kind, in the order the total line gives them.
    pub const ALL: [Proof; 4] = [Proof::Checked, Proof::Judged, Proof::Person, Proof::Claim];

    /// The proof of `step`, closed `done`.
    pub fn of(step: Step, done: &Done) -> Proof {
        match (done.by, &step.record().check) {
            (Some(By::Judge), _) => Proof::Judged,
            (Some(By::Person), _) => Proof::Person,
            (None, Some(_)) => Proof::Checked,
            (None, None) => Proof::Claim,
        }
    }

    /// Its word for `n` steps.
    pub fn word(self, n: usize) -> &'static str {
        match (self, n) {
            (Proof::Checked, _) => "checked",
            (Proof::Judged, _) => "judged",
            (Proof::Person, _) => "person",
            (Proof::Claim, 1) => "claim",
            (Proof::Claim, _) => "claims",
        }
    }
}
