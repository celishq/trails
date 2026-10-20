use super::*;

/// An OSC 11 answer gives the background, each part cut to its first two hex digits, BEL or ST at its
/// end; anything else gives none.
#[test]
fn an_osc_11_answer_gives_the_background() {
    assert_eq!(
        parse(b"\x1b]11;rgb:2323/2626/3131\x07"),
        Some((0x23, 0x26, 0x31))
    );
    assert_eq!(parse(b"\x1b]11;rgb:06/0e/1e\x1b\\"), Some((6, 14, 30)));
    assert_eq!(parse(b""), None);
    assert_eq!(parse(b"\x1b]11;rgb:zz/00/00\x07"), None);
}

/// Before the terminal answered, the background is the fallback.
#[test]
fn the_background_falls_back_when_none_was_said() {
    if SAID.get().is_none() {
        assert_eq!(background(), FALLBACK);
    }
}
