//! How a view drawn with ratatui prints: its looks, its lines of text, and a line as text.

use ratatui::buffer::Buffer;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// A line of text with its looks.
pub(crate) type Styled = Line<'static>;

pub(crate) const DIM: Style = Style::new().add_modifier(Modifier::DIM);
pub(crate) const BOLD: Style = Style::new().add_modifier(Modifier::BOLD);
/// Done: a neon mint.
pub(crate) const GREEN: Style = Style::new().fg(Color::Rgb(52, 230, 140));
/// Changed from its plan: amber.
pub(crate) const YELLOW: Style = Style::new().fg(Color::Rgb(250, 204, 21));
/// Working: a neon cyan.
pub(crate) const CYAN: Style = Style::new().fg(Color::Rgb(34, 211, 238));
/// Rules: a neon violet.
pub(crate) const MAGENTA: Style = Style::new().fg(Color::Rgb(192, 96, 255));
/// Dropped: rose.
pub(crate) const RED: Style = Style::new().fg(Color::Rgb(251, 113, 133));
/// The border of a day's box and the rule under its line: a slate blue, a little lighter than the
/// screen.
pub(crate) const FRAME: Style = Style::new().fg(Color::Rgb(30, 48, 76));
/// The border of celis's card and the line after its headline, lighter than a day's box.
pub(crate) const EDGE: Style = Style::new().fg(Color::Rgb(44, 66, 104));
/// A second share beside cyan, like a second project's time: a quiet slate.
pub(crate) const SLATE: Style = Style::new().fg(Color::Rgb(86, 120, 168));
/// What the person selected: the text's light.
pub(crate) const LIGHT: Style = Style::new().fg(Color::Rgb(241, 245, 249));

/// `text` in the look `style`.
pub(crate) fn paint(style: Style, text: &str) -> Span<'static> {
    Span::styled(text.to_string(), style)
}

/// The cells of `text` a terminal shows: a wide character counts 2, a narrow one an emoji selector
/// makes wide counts 2 too, a colour code none.
pub(crate) fn visible(text: &str) -> usize {
    let mut count = 0;
    let mut code = false;
    let mut before = 0;
    for c in text.chars() {
        match (code, c) {
            (false, '\x1b') => code = true,
            (true, 'm') => code = false,
            (false, '\u{fe0f}') if before == 1 => {
                count += 1;
                before = 2;
            }
            (false, c) => {
                before = cells(c);
                count += before;
            }
            _ => {}
        }
    }
    count
}

/// The cells `c` takes on a terminal.
fn cells(c: char) -> usize {
    c.width().unwrap_or(0)
}

/// `text` cut to `width` visible cells, its last one `…`, when it is wider.
pub(crate) fn cut(text: &str, width: usize) -> String {
    if width < 2 || visible(text) <= width {
        return text.to_string();
    }
    let mut kept = String::new();
    for c in text.chars() {
        if visible(&kept) + cells(c) > width - 1 {
            break;
        }
        kept.push(c);
    }
    kept.push('…');
    if text.contains('\x1b') {
        kept.push_str("\x1b[0m");
    }
    kept
}

/// The cells of `line` on a terminal.
pub(crate) fn width(line: &Styled) -> usize {
    line.spans.iter().map(|s| s.content.width()).sum()
}

/// `line` cut to `width` cells, its last one `…` in the look before it, when it is wider.
pub(crate) fn clip(line: &Styled, width: usize) -> Styled {
    if width < 2 || self::width(line) <= width {
        return line.clone();
    }
    let mut left = width - 1;
    let mut kept: Vec<Span<'static>> = Vec::new();
    for span in &line.spans {
        if left == 0 {
            break;
        }
        let mut text = String::new();
        for c in span.content.chars() {
            if cells(c) > left {
                left = 0;
                break;
            }
            left -= cells(c);
            text.push(c);
        }
        kept.push(Span::styled(text, span.style));
    }
    match kept.last_mut() {
        Some(last) => last.content.to_mut().push('…'),
        None => kept.push(Span::raw("…")),
    }
    Styled::from(kept)
}

/// `line` with spaces after it to `width` cells.
pub(crate) fn pad(mut line: Styled, width: usize) -> Styled {
    let pad = width.saturating_sub(self::width(&line));
    if pad > 0 {
        line.push_span(" ".repeat(pad));
    }
    line
}

/// The spans of `lines`, one line after another.
pub(crate) fn join(lines: impl IntoIterator<Item = Styled>) -> Styled {
    Styled::from(lines.into_iter().flat_map(|l| l.spans).collect::<Vec<_>>())
}

/// The rows of `buf` as lines, a span for each run of one look.
pub(crate) fn lines(buf: &Buffer) -> Vec<Styled> {
    let area = buf.area;
    let row = |y| {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut skip = 0;
        for x in area.left()..area.right() {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            let cell = &buf[(x, y)];
            skip = cell.symbol().width().saturating_sub(1);
            let style = Style::new()
                .fg(cell.fg)
                .bg(cell.bg)
                .add_modifier(cell.modifier);
            match spans.last_mut() {
                Some(last) if last.style == style => last.content.to_mut().push_str(cell.symbol()),
                _ => spans.push(Span::styled(cell.symbol().to_string(), style)),
            }
        }
        Styled::from(spans)
    };
    (area.top()..area.bottom()).map(row).collect()
}

/// `line` as text; when `colour`, each run of one look, the line's look under its own, in its escape
/// code and a reset after.
pub(crate) fn line(line: &Styled, colour: bool) -> String {
    let mut runs: Vec<(Option<String>, String)> = Vec::new();
    for span in &line.spans {
        let codes = colour
            .then(|| codes(line.style.patch(span.style), graphic(&span.content)))
            .flatten();
        match runs.last_mut() {
            Some((last, text)) if *last == codes => *text += &span.content,
            _ => runs.push((codes, span.content.to_string())),
        }
    }
    let run = |(codes, text): (Option<String>, String)| match codes {
        Some(codes) => format!("\x1b[{codes}m{text}\x1b[0m"),
        None => text,
    };
    runs.into_iter().map(run).collect()
}

/// Whether `text` draws lines and blocks, not words: box and block characters, `■` and spaces only.
fn graphic(text: &str) -> bool {
    let drawn = |c: char| ('\u{2500}'..='\u{259F}').contains(&c) || c == '■';
    text.chars().any(drawn) && text.chars().all(|c| c == ' ' || drawn(c))
}

/// The escape codes of `style` as it reads on the terminal's background, a `graphic`'s or a text's,
/// its looks then its colours; `None` for the plain look.
fn codes(style: Style, graphic: bool) -> Option<String> {
    let style = super::contrast::legible(style, super::background::background(), graphic);
    let looks = [
        (Modifier::BOLD, "1"),
        (Modifier::DIM, "2"),
        (Modifier::ITALIC, "3"),
        (Modifier::UNDERLINED, "4"),
        (Modifier::REVERSED, "7"),
        (Modifier::CROSSED_OUT, "9"),
    ];
    let modifier = style.add_modifier;
    let mut codes: Vec<String> = looks
        .iter()
        .filter(|(m, _)| modifier.contains(*m))
        .map(|(_, c)| c.to_string())
        .collect();
    codes.extend(style.fg.filter(|c| *c != Color::Reset).map(|c| code(c, 30)));
    codes.extend(style.bg.filter(|c| *c != Color::Reset).map(|c| code(c, 40)));
    (!codes.is_empty()).then(|| codes.join(";"))
}

/// The code of `colour` from `base`, 30 for the text and 40 for the background.
fn code(colour: Color, base: u8) -> String {
    let named = |n: u8| (base + n).to_string();
    let bright = |n: u8| (base + 60 + n).to_string();
    match colour {
        Color::Reset => named(9),
        Color::Black => named(0),
        Color::Red => named(1),
        Color::Green => named(2),
        Color::Yellow => named(3),
        Color::Blue => named(4),
        Color::Magenta => named(5),
        Color::Cyan => named(6),
        Color::Gray => named(7),
        Color::DarkGray => bright(0),
        Color::LightRed => bright(1),
        Color::LightGreen => bright(2),
        Color::LightYellow => bright(3),
        Color::LightBlue => bright(4),
        Color::LightMagenta => bright(5),
        Color::LightCyan => bright(6),
        Color::White => bright(7),
        Color::Indexed(n) => format!("{};5;{n}", base + 8),
        Color::Rgb(r, g, b) => format!("{};2;{r};{g};{b}", base + 8),
    }
}

#[cfg(test)]
mod tests;
