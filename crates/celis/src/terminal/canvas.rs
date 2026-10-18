//! How a view drawn with ratatui prints: its looks, its lines of text, and its buffer or one line as
//! text.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthStr;

/// A line of text with its looks.
pub(crate) type Styled = Line<'static>;

pub(crate) const DIM: Style = Style::new().add_modifier(Modifier::DIM);
pub(crate) const BOLD: Style = Style::new().add_modifier(Modifier::BOLD);
pub(crate) const GREEN: Style = Style::new().fg(Color::Green);
pub(crate) const YELLOW: Style = Style::new().fg(Color::Yellow);
pub(crate) const MAGENTA: Style = Style::new().fg(Color::Magenta);
pub(crate) const RED: Style = Style::new().fg(Color::Red);

/// `text` in the look `style` when `colour`; else plain.
pub(crate) fn paint(colour: bool, style: Style, text: &str) -> Span<'static> {
    match colour {
        true => Span::styled(text.to_string(), style),
        false => Span::raw(text.to_string()),
    }
}

/// The characters of `line`.
pub(crate) fn width(line: &Styled) -> usize {
    line.spans.iter().map(|s| s.content.chars().count()).sum()
}

/// `line` cut to `width` characters, its last one `…` in the look before it, when it is wider.
pub(crate) fn cut(line: &Styled, width: usize) -> Styled {
    if width < 2 || self::width(line) <= width {
        return line.clone();
    }
    let mut left = width - 1;
    let mut kept: Vec<Span<'static>> = Vec::new();
    for span in &line.spans {
        if left == 0 {
            break;
        }
        let text: String = span.content.chars().take(left).collect();
        left -= text.chars().count();
        kept.push(Span::styled(text, span.style));
    }
    match kept.last_mut() {
        Some(last) => last.content.to_mut().push('…'),
        None => kept.push(Span::raw("…")),
    }
    Styled::from(kept)
}

/// `line` with spaces after it to `width` characters.
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

/// The rows of `buf` as text; when `colour`, each run of one look in its escape code and a reset after.
pub(crate) fn rows(buf: &Buffer, colour: bool) -> Vec<String> {
    let area = buf.area;
    let row = |y| {
        let mut runs: Vec<(Style, String)> = Vec::new();
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
            match runs.last_mut() {
                Some((last, text)) if *last == style => *text += cell.symbol(),
                _ => runs.push((style, cell.symbol().to_string())),
            }
        }
        let run = |(style, text): (Style, String)| match (colour, codes(style)) {
            (true, Some(codes)) => format!("\x1b[{codes}m{text}\x1b[0m"),
            _ => text,
        };
        runs.into_iter().map(run).collect()
    };
    (area.top()..area.bottom()).map(row).collect()
}

/// `line` as text, as wide as it is; with no `colour`, its text as it is.
pub(crate) fn line(line: &Line, colour: bool) -> String {
    if !colour {
        return line.spans.iter().map(|s| s.content.as_ref()).collect();
    }
    let width = u16::try_from(line.width()).unwrap_or(u16::MAX);
    if width == 0 {
        return String::new();
    }
    let mut buf = Buffer::empty(Rect::new(0, 0, width, 1));
    line.render(buf.area, &mut buf);
    rows(&buf, colour).remove(0)
}

/// The escape codes of `style`, its looks then its colours; `None` for the plain look.
fn codes(style: Style) -> Option<String> {
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
