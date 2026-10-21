use super::*;
use ratatui::text::Span;

/// A line prints each run of one look in its escape code with a reset after, and plain text plain.
#[test]
fn a_line_prints_each_run_in_its_escape_code() {
    let line = Line::from(vec![
        Span::styled("✓", Style::new().fg(Color::Green)),
        Span::raw(" 1 closed step "),
        Span::styled(
            "§",
            Style::new().add_modifier(Modifier::BOLD).fg(Color::Magenta),
        ),
    ]);
    assert_eq!(
        super::line(&line, true),
        "\x1b[32m✓\x1b[0m 1 closed step \x1b[1;35m§\x1b[0m"
    );
    assert_eq!(super::line(&line, false), "✓ 1 closed step §");
}

/// A line keeps a character the terminal shows as one cell, as a placeholder for the spinner.
#[test]
fn a_line_keeps_a_private_character() {
    let line = Line::from(Span::styled(
        "\u{E000}",
        Style::new().fg(Color::LightYellow),
    ));
    assert_eq!(super::line(&line, true), "\x1b[93m\u{E000}\x1b[0m");
}

/// A line cut to a width keeps its looks, and its last character is `…` in the look before it.
#[test]
fn a_cut_line_keeps_its_looks() {
    let line = Styled::from(vec![Span::styled("ab", BOLD), Span::raw("cdef")]);
    let cut = clip(&line, 4);
    assert_eq!(cut.to_string(), "abc…");
    assert_eq!(cut.spans[1].style, Style::new());
    assert_eq!(clip(&line, 2).spans, [Span::styled("a…", BOLD)]);
    assert_eq!(clip(&line, 6), line);
}

/// A wide character takes two cells: it counts 2, and a cut keeps no more cells than its width.
#[test]
fn a_wide_character_counts_two_cells() {
    assert_eq!(visible("日本 ok"), 7);
    assert_eq!(width(&Styled::raw("日本")), 4);
    assert_eq!(cut("日本語の道", 6), "日本…");
    assert_eq!(clip(&Styled::raw("日本語の道"), 6).to_string(), "日本…");
}

/// A line's own look reaches each of its runs.
#[test]
fn a_line_prints_its_own_look_on_each_run() {
    let line = Styled::styled("Usage", BOLD);
    assert_eq!(super::line(&line, true), "\x1b[1mUsage\x1b[0m");
}
