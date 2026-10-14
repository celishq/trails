//! How the work of one closed step went.

use serde_json::{json, Value};

/// How the work of one closed step went: the seconds it took, and the steps that replaced it after.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grade {
    pub took: u64,
    pub revised: Vec<String>,
}

impl Grade {
    /// The text line printed under a done step.
    pub fn line(&self) -> String {
        let by: Vec<String> = self.revised.iter().map(|id| crate::id::short(id)).collect();
        match by.is_empty() {
            true => format!("took {}", took(self.took)),
            false => format!("took {} · revised by {}", took(self.took), by.join(", ")),
        }
    }

    /// The `--json` value of this grade.
    pub fn value(&self) -> Value {
        json!({"took": self.took, "revised": self.revised})
    }
}

/// `seconds` in its two largest units.
pub fn took(seconds: u64) -> String {
    match seconds {
        s if s < 60 => format!("{s}s"),
        s if s < 3_600 => format!("{}m {}s", s / 60, s % 60),
        s if s < 86_400 => format!("{}h {}m", s / 3_600, s % 3_600 / 60),
        s => format!("{}d {}h", s / 86_400, s % 86_400 / 3_600),
    }
}

/// `seconds` as `took` gives it, with a `-` when below zero.
pub fn saved(seconds: i64) -> String {
    let sign = if seconds < 0 { "-" } else { "" };
    format!("{sign}{}", took(seconds.unsigned_abs()))
}

/// The seconds until `took` shows another value for `seconds`.
pub fn stride(seconds: u64) -> u64 {
    match seconds {
        s if s < 3_600 => 1,
        s if s < 86_400 => 60 - s % 60,
        s => 3_600 - s % 3_600,
    }
}

#[cfg(test)]
mod tests;
