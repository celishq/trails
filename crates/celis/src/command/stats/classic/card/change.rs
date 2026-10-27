//! How a number of a card moved from the 14 days before to the last 14.

/// A number of a card in the 14 days before and in the last 14.
pub(super) struct Change {
    pub before: i64,
    pub last: i64,
}

impl Change {
    /// `▲ <by>` when it rose, `▼ <by>` when it fell, `=` when it held.
    pub fn arrow(&self) -> String {
        let moved = self.last - self.before;
        match moved.signum() {
            0 => "=".to_string(),
            1 => format!("▲ {moved}"),
            _ => format!("▼ {}", -moved),
        }
    }
}
