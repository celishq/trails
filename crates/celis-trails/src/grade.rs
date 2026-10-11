//! How the work of one closed step went.

use serde_json::{json, Value};

/// How the work of one closed step went: the seconds it took, the steps that fix it, and the steers that
/// name it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grade {
    pub took: u64,
    pub rework: usize,
    pub steered: usize,
}

impl Grade {
    /// The text line printed under a done step.
    pub fn line(&self) -> String {
        format!(
            "took {} · rework {} · steered {}",
            took(self.took),
            self.rework,
            self.steered
        )
    }

    /// The `--json` value of this grade.
    pub fn value(&self) -> Value {
        json!({"took": self.took, "rework": self.rework, "steered": self.steered})
    }
}

/// `seconds` in its two largest units.
pub(crate) fn took(seconds: u64) -> String {
    match seconds {
        s if s < 60 => format!("{s}s"),
        s if s < 3_600 => format!("{}m {}s", s / 60, s % 60),
        s if s < 86_400 => format!("{}h {}m", s / 3_600, s % 3_600 / 60),
        s => format!("{}d {}h", s / 86_400, s % 86_400 / 3_600),
    }
}

/// The seconds until `took` shows another value for `seconds`.
pub(crate) fn stride(seconds: u64) -> u64 {
    match seconds {
        s if s < 3_600 => 1,
        s if s < 86_400 => 60 - s % 60,
        s => 3_600 - s % 3_600,
    }
}

#[cfg(test)]
mod tests;
