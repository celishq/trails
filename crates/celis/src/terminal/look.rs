//! How a step looks in every view: the mark and looks of its state, the signs of its flags, and the
//! spinner of a working one.

use super::canvas::{Styled, BOLD, CYAN, DIM, GREEN, RED, YELLOW};
use crate::progress::Flags;
use celis_trails::State;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use std::time::Duration;

/// How a step in a state prints: its mark, the look of its mark, and the look of its title.
pub(crate) struct Look {
    pub mark: &'static str,
    pub text: Style,
    pub title: Style,
}

/// How a step or a trail in `state` prints.
pub(crate) fn look(state: State) -> Look {
    let (mark, text, title) = match state {
        State::Open => ("○", Style::new(), Style::new()),
        State::Working => ("◐", CYAN, BOLD),
        State::Done => ("✓", GREEN, Style::new()),
        State::Dropped => ("✗", RED, DIM),
        State::Expired => ("◌", DIM, DIM),
    };
    Look { mark, text, title }
}

/// The look of a lane that hangs from a step or trail in `state`: its state's colour, the spinner's
/// while working, dim while open or expired.
pub(crate) fn lane(state: State) -> Style {
    match state {
        State::Working => SPINNER,
        State::Done => GREEN,
        State::Dropped => RED,
        State::Open | State::Expired => DIM,
    }
}

/// The signs of `flags` with their looks: `↻` when revised, then `~` when changed.
pub(crate) fn signs(flags: Flags) -> Vec<(&'static str, Style)> {
    let all = [(flags.revised, "↻", GREEN), (flags.changed, "~", YELLOW)];
    all.into_iter()
        .filter(|(on, ..)| *on)
        .map(|(_, sign, style)| (sign, style))
        .collect()
}

/// The frames of the spinner of a working step.
const SPIN: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// What stands for the spinner in a text until a view draws its frame.
pub(crate) const SPUN: &str = "\u{E000}";

/// What stands for a check being drawn in a text until a view draws its stroke.
pub(crate) const DRAWN: &str = "\u{E001}";

/// The strokes of a check being drawn, one a frame, before the check itself.
pub(crate) const STROKES: [&str; 5] = ["⠄", "⢄", "⢤", "⢴", "⢼"];

/// The mark on a title that shimmers in a view: a bold band runs across it.
pub(crate) const SHIMMER: Modifier = Modifier::SLOW_BLINK;

/// The characters of the band that runs across a shimmering title.
const BAND: usize = 4;

/// The look of the spinner: the working cyan.
pub(crate) const SPINNER: Style = CYAN;

/// The time between two frames of the spinner.
pub(crate) const TURN: Duration = Duration::from_millis(100);

/// The spinner at its frame `frame`.
pub(crate) fn frame(frame: usize) -> &'static str {
    SPIN[frame % SPIN.len()]
}

/// `lines` with each spinner in them at its frame `frame`, each check being drawn at its stroke
/// `drawn` frames after it was first shown, then `✓`, and a bold band across each shimmering title.
pub(crate) fn spinner(lines: &[Styled], frame: usize, drawn: usize) -> Vec<Styled> {
    let stroke = STROKES.get(drawn).copied().unwrap_or("✓");
    let mut lines = lines.to_vec();
    for line in &mut lines {
        let mut spans = vec![];
        for mut span in std::mem::take(&mut line.spans) {
            if span.content.contains(SPUN) || span.content.contains(DRAWN) {
                let content = span.content.replace(SPUN, self::frame(frame));
                span.content = content.replace(DRAWN, stroke).into();
            }
            match span.style.add_modifier.contains(SHIMMER) {
                true => spans.extend(shimmer(
                    &span.content,
                    span.style.remove_modifier(SHIMMER),
                    frame,
                )),
                false => spans.push(span),
            }
        }
        line.spans = spans;
    }
    lines
}

/// `text` in `style` with a bold band across it at the spinner's frame `frame`, two characters a frame,
/// a pause between runs.
fn shimmer(text: &str, style: Style, frame: usize) -> Vec<Span<'static>> {
    let chars: Vec<char> = text.chars().collect();
    let run = chars.len() + 3 * BAND;
    let head = (frame * 2) % run;
    let start = head.saturating_sub(BAND).min(chars.len());
    let end = head.min(chars.len());
    let part = |from: usize, to: usize| chars[from..to].iter().collect::<String>();
    let parts = [
        (part(0, start), style),
        (part(start, end), style.add_modifier(Modifier::BOLD)),
        (part(end, chars.len()), style),
    ];
    parts
        .into_iter()
        .filter(|(text, _)| !text.is_empty())
        .map(|(text, style)| Span::styled(text, style))
        .collect()
}

/// The spinner's frame at this moment.
pub(crate) fn spin() -> usize {
    let since = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    since.map_or(0, |d| (d.as_millis() / TURN.as_millis()) as usize)
}

#[cfg(test)]
mod tests;
