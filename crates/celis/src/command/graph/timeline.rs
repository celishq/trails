//! The timeline of the node view: where the replay stands among the events of a trail.

use super::events::{Events, Kind};
use crate::command::tree::cut;
use crate::terminal::canvas::{BOLD, CYAN, DIM, GREEN};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

/// Draws the bottom two rows of `area` on `buf`: whether it plays, follows or stands at an event after
/// the first `shown` of `events`, with the newest event shown, then a bar of the events shown.
pub(super) fn timeline(
    events: &Events,
    shown: usize,
    (playing, live): (bool, bool),
    area: Rect,
    buf: &mut Buffer,
) {
    let all = events.all.len();
    let (status, colour) = match (playing, live) {
        (true, _) => (format!("▶ {}/{all}", shown), CYAN),
        (false, true) => ("● live".to_string(), GREEN),
        (false, false) => (format!("⏸ {}/{all}", shown), DIM),
    };
    let style = colour.patch(BOLD);
    let width = usize::from(area.width);
    buf.set_string(area.x, area.y, &status, style);
    if let Some(event) = events.all[..shown].last() {
        let said = format!("{} {}", word(event.kind), event.text);
        let room = width.saturating_sub(status.chars().count() + 3);
        let x = area.x + status.chars().count() as u16 + 3;
        buf.set_string(x, area.y, cut(&said, room), Style::new());
    }
    let at = match all {
        0 => 0,
        all => shown * width.saturating_sub(1) / all,
    };
    for x in 0..width {
        let (bar, colour) = match x.cmp(&at) {
            std::cmp::Ordering::Less => ("━", CYAN),
            std::cmp::Ordering::Equal => ("●", BOLD),
            std::cmp::Ordering::Greater => ("─", DIM),
        };
        buf.set_string(area.x + x as u16, area.y + 1, bar, colour);
    }
}

/// The word the timeline says for an event of `kind`.
fn word(kind: Kind) -> &'static str {
    match kind {
        Kind::Added => "added",
        Kind::Rule => "§ rule",
        Kind::Started => "started",
        Kind::Done => "done",
        Kind::Dropped => "dropped",
    }
}
