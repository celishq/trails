use super::*;
use celis_nostr::Note;

/// A window holds the last 14 days up to its time, the 14 days before them, and 14 days, oldest
/// first, the last one today.
#[test]
fn a_window_holds_two_spans_and_14_days() {
    let now = 100 * DAY + 3_600;
    let records: Vec<Note> = vec![];
    let steps = Steps::new(&records, now);
    let window = Window::new(&steps);
    assert_eq!(window.last.from(), now - DAYS * DAY);
    assert_eq!(window.before.from(), now - 2 * DAYS * DAY);
    assert_eq!(window.days.len(), DAYS as usize);
    assert_eq!(window.days.last().unwrap().from(), midnight(now));
    assert_eq!(window.days[0].from(), midnight(now) - 13 * DAY);
}
