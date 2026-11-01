use super::{drawn, inline};

#[test]
fn inline_draws_the_bytes_named_in_iterm2_form() {
    assert_eq!(
        inline("logo.png", b"abc"),
        "\x1b]1337;File=name=bG9nby5wbmc=;size=3;inline=1:YWJj\x07"
    );
}

#[test]
fn drawn_only_on_a_terminal_that_draws_inline_images() {
    assert!(drawn(true, Some("iTerm.app")));
    assert!(drawn(true, Some("WezTerm")));
    assert!(!drawn(false, Some("iTerm.app")));
    assert!(!drawn(true, Some("Apple_Terminal")));
    assert!(!drawn(true, None));
}
