use super::*;
use crate::terminal::remark::Kind;

/// The card shows each new value once, oldest first, holds it a while, and never shows one again: once
/// none is new, it keeps the last.
#[test]
fn the_card_shows_each_value_once() {
    let mut screen = Screen::default();
    let said = [
        Remark::new(Kind::Kept, "the newer", 2),
        Remark::new(Kind::Learned, "the older", 1),
    ];
    let shows = |screen: &mut Screen| {
        let shown = turned(screen, &said);
        screen.remark = shown.clone().map(|r| (r, None));
        screen.since = screen.since.map(|_| Instant::now() - HOLD);
        shown.map(|r| r.at)
    };
    assert_eq!(shows(&mut screen), Some(1));
    assert_eq!(shows(&mut screen), Some(2));
    assert_eq!(shows(&mut screen), Some(2));
    assert_eq!(screen.seen.len(), 2);
}
