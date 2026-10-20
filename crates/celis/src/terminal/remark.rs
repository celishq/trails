//! What celis says above a live tree: the newest value it brought a trail, as a reply beside the dude.

use super::canvas::{cut, paint, visible, Styled, EDGE};
use super::dude::{dude, COLUMNS};
use super::unfold::toward;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

/// The rows a remark takes above a tree: its box's six and a blank line.
pub(crate) const ROWS: usize = 7;

/// What celis says, of a `kind`, in `words`, each strong or not, since `at`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Remark {
    pub kind: Kind,
    pub words: Vec<(String, bool)>,
    pub at: u64,
}

/// What a remark tells: a rule learned, a pitfall kept, a rule kept, past work reused, a trail done, or
/// how the trail stands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Kind {
    Learned,
    Pitfall,
    Kept,
    Reused,
    Done,
    Standing,
}

/// The headline's colour of what celis learned or kept: the rules' violet.
const VIOLET: Color = Color::Rgb(192, 96, 255);

/// The headline's colour of reused work: the done mint.
const MINT: Color = Color::Rgb(52, 230, 140);

/// The headline's colour of how a trail stands: a quiet slate.
const SLATE: Color = Color::Rgb(148, 163, 184);

/// The caret's colour: the text's own grey.
const CARET: Color = Color::Rgb(148, 163, 184);

impl Kind {
    /// Its emoji, headline and colour.
    fn head(self) -> (&'static str, &'static str, Color) {
        match self {
            Kind::Learned => ("💡", "Learned", VIOLET),
            Kind::Pitfall => ("🩹", "Pitfall kept", VIOLET),
            Kind::Kept => ("🎉", "Rule paid off", VIOLET),
            Kind::Reused => ("🔁", "Reused past work", MINT),
            Kind::Done => ("🏆", "Trail done", VIOLET),
            Kind::Standing => ("📈", "This trail so far", SLATE),
        }
    }
}

impl Remark {
    /// A remark of `kind` at `at` from `text`, its strong words between `**` and `**`: each word with
    /// the space before it, none where it follows the word before with none.
    pub fn new(kind: Kind, text: &str, at: u64) -> Remark {
        let mut words: Vec<(String, bool)> = vec![];
        let mut spaced = false;
        for (n, part) in text.split("**").enumerate() {
            spaced |= part.starts_with(char::is_whitespace);
            for word in part.split_whitespace() {
                let gap = if spaced && !words.is_empty() { " " } else { "" };
                words.push((format!("{gap}{word}"), n % 2 == 1));
                spaced = true;
            }
            spaced = part.ends_with(char::is_whitespace);
        }
        Remark { kind, words, at }
    }

    /// The characters of its words, the spaces between them counted.
    pub fn length(&self) -> usize {
        self.words
            .iter()
            .map(|(word, _)| word.chars().count())
            .sum()
    }

    /// Its rows at the spinner's frame `tick`, `width` wide: one rounded blue box holding the dude, a rule
    /// reaching toward its emoji and headline bold in its kind's colour, `shown` of the way from the
    /// screen's background, with a line after them, a blank row, and its words italic in two lines, the
    /// second cut with `…`; only the first `typed` characters of the words when some, a grey `caret`
    /// after them when on; then a blank line.
    pub fn rows(
        &self,
        tick: usize,
        (typed, caret, shown): (Option<usize>, bool, f64),
        width: usize,
    ) -> Vec<Styled> {
        let (emoji, headline, colour) = self.kind.head();
        let size = width.saturating_sub(2).max(40);
        let inner = size - (8 + COLUMNS);
        let head = format!("{emoji} {headline} ");
        let line = (inner / 2).saturating_sub(visible(&head)).max(3);
        let mut texts = vec![vec![
            paint(EDGE, "─"),
            Span::raw(" "),
            Span::styled(
                head.clone(),
                Style::new()
                    .fg(toward(colour, shown))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("─".repeat(line.saturating_sub(1)), EDGE),
            Span::raw(" ".repeat(inner.saturating_sub(visible(&head) + line) + 1)),
        ]];
        let mut left = typed.unwrap_or(usize::MAX);
        let mut placed = !caret;
        texts.push(vec![Span::raw(" ".repeat(inner + 2))]);
        let lines: Vec<String> = lines(&self.words, inner)
            .into_iter()
            .map(|line| line.into_iter().map(|(word, _)| word).collect())
            .chain(std::iter::repeat(String::new()))
            .take(2)
            .collect();
        for (n, line) in lines.iter().enumerate() {
            let count = line.chars().count();
            let shown: String = line.chars().take(left).collect();
            left = left.saturating_sub(count);
            let here = !placed && (left == 0 || n == 1 || lines[n + 1].is_empty());
            let mut spans = vec![
                Span::raw(" "),
                Span::styled(shown.clone(), Style::new().add_modifier(Modifier::ITALIC)),
            ];
            let mut used = visible(&shown);
            if here {
                spans.push(Span::styled("▍", Style::new().fg(CARET)));
                used += 1;
                placed = true;
            }
            spans.push(Span::raw(" ".repeat(inner.saturating_sub(used) + 1)));
            texts.push(spans);
        }
        let edge = |left: &str, right: &str| {
            let border = format!("{left}{}{right}", "─".repeat(size - 2));
            Styled::from(vec![Span::raw("  "), paint(EDGE, &border)])
        };
        let mut rows = vec![edge("╭", "╮")];
        let body = dude(tick)
            .into_iter()
            .chain(std::iter::repeat(Styled::from(" ".repeat(COLUMNS))));
        for (n, (body, text)) in body.zip(texts).enumerate() {
            let mut row = Styled::from(vec![Span::raw("  "), paint(EDGE, "│ ")]);
            row.spans.extend(body.spans);
            row.spans.push(Span::raw("  "));
            row.spans.push(paint(EDGE, if n == 0 { "├" } else { "│" }));
            row.spans.extend(text);
            row.spans.push(paint(EDGE, "│"));
            rows.push(row);
        }
        rows.push(edge("╰", "╯"));
        rows.push(Styled::default());
        rows
    }
}

/// `words` in two lines of at most `room` cells, broken before a word with a space before it; the second
/// cut with `…` when words are left.
fn lines(words: &[(String, bool)], room: usize) -> Vec<Vec<(String, bool)>> {
    let mut lines: Vec<Vec<(String, bool)>> = vec![vec![]];
    let mut used = 0;
    for (word, bold) in words {
        let mut word = word.clone();
        if used + visible(&word) > room && used > 0 && word.starts_with(' ') {
            if lines.len() == 2 {
                let line = lines.last_mut().unwrap_or_else(|| unreachable!());
                while used + 1 > room {
                    let Some((last, _)) = line.pop() else { break };
                    used -= visible(&last);
                }
                line.push(("…".to_string(), false));
                return lines;
            }
            lines.push(vec![]);
            word = word.trim_start().to_string();
            used = 0;
        }
        let word = cut(&word, room.saturating_sub(used));
        used += visible(&word);
        lines
            .last_mut()
            .unwrap_or_else(|| unreachable!())
            .push((word, *bold));
    }
    lines
}

#[cfg(test)]
mod tests;
