//! One frame of the graph view: what it saw, the moment it draws, and the looks of steps at that
//! moment.

use super::seen::Seen;
use crate::progress::Phase;
use crate::terminal::canvas::{paint, CYAN, DIM, GREEN, RED, YELLOW};
use crate::terminal::look::{frame, STROKES};
use crate::terminal::unfold::{ease, mix, toward};
use celis_trails::Step;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use std::collections::HashSet;
use std::time::Instant;

/// The text's light, for what the person selected.
pub(super) const LIGHT: Color = match crate::terminal::canvas::LIGHT.fg {
    Some(light) => light,
    None => Color::Reset,
};

/// The shade behind a row or a trail the person selected.
pub(super) const SEL: Color = Color::Rgb(51, 65, 98);

/// The light a check glows with as it settles.
const MINT: Color = Color::Rgb(210, 255, 225);

/// The light a pattern glows with as it lands.
const LILAC: Color = Color::Rgb(255, 210, 255);

/// The light a sign glows with as it lands.
const CREAM: Color = Color::Rgb(255, 255, 230);

/// The moment a frame draws: what the view saw, now, the seconds since the view opened, and the
/// nestings the person unfolded.
pub(super) struct Moment<'a> {
    pub seen: &'a Seen,
    pub now: Instant,
    pub unfolded: &'a HashSet<String>,
    pub at: u64,
}

impl Moment<'_> {
    /// The seconds since the view opened.
    pub fn t(&self) -> f64 {
        self.now.duration_since(self.seen.opened).as_secs_f64()
    }

    /// The spinner's frame now.
    pub fn spin(&self) -> &'static str {
        frame((self.t() * 10.0) as usize)
    }

    /// The mark of `step` now: the spinner while it works; as it is done, the spinner fading out, then
    /// a check drawn stroke by stroke, then `✓` with a short glow; `✗` dropped, `◌` stale, `○` open.
    pub fn mark(&self, step: Step) -> (&'static str, Style) {
        let since = self.seen.changed(step.id(), self.now).unwrap_or(9.0);
        match Phase::of(&step) {
            Phase::Started => (self.spin(), CYAN),
            Phase::Done if since < 0.15 => {
                let fade = toward(rgb(CYAN), 1.0 - since / 0.15);
                (self.spin(), Style::new().fg(fade))
            }
            Phase::Done if since < 0.45 => {
                let stroke = ((since - 0.15) / 0.06) as usize;
                (STROKES[stroke.min(4)], GREEN)
            }
            Phase::Done => {
                let glow = 1.0 - (since - 0.45) / 0.9;
                ("✓", Style::new().fg(mix(rgb(GREEN), MINT, glow)))
            }
            Phase::Dropped => ("✗", RED),
            Phase::Stale => ("◌", DIM),
            Phase::Open => ("○", Style::new()),
        }
    }

    /// `˜` in amber beside the mark of a step done otherwise than planned, glowing once as it lands.
    pub fn sign(&self, step: Step) -> Option<Span<'static>> {
        if Phase::of(&step) != Phase::Done || step.changed().is_none() {
            return None;
        }
        let since = self.seen.changed(step.id(), self.now).unwrap_or(9.0);
        let glow = 1.0 - (since - 0.45) / 1.2;
        Some(paint(Style::new().fg(mix(rgb(YELLOW), CREAM, glow)), "˜"))
    }

    /// The look of the title of a step in `state`: dim open, dropped or stale.
    pub fn tone(&self, state: Phase) -> Style {
        match state {
            Phase::Done | Phase::Started => Style::new(),
            _ => DIM,
        }
    }

    /// The colour of the border of the card of `step`: its state's, pulsing while it works and glowing
    /// once as it is done; light when `selected`.
    pub fn border(&self, step: Step, selected: bool) -> Style {
        if selected {
            return Style::new().fg(LIGHT);
        }
        let colour = match Phase::of(&step) {
            Phase::Started => {
                let pulse = 0.5 + 0.5 * (self.t() * 3.2).sin();
                mix(toward(rgb(CYAN), 0.6), rgb(CYAN), pulse)
            }
            Phase::Done => {
                let since = self.seen.changed(step.id(), self.now).unwrap_or(9.0);
                mix(toward(rgb(GREEN), 0.7), MINT, 1.0 - (since - 0.45) / 0.9)
            }
            Phase::Dropped => toward(rgb(RED), 0.5),
            Phase::Open | Phase::Stale => return DIM,
        };
        Style::new().fg(colour)
    }

    /// The glow of a pattern that landed `since` seconds ago, magenta settling from lilac.
    pub fn pattern(&self, since: Option<f64>) -> Style {
        let glow = since.map_or(0.0, |s| 1.0 - s / 1.4);
        Style::new().fg(mix(rgb(crate::terminal::canvas::MAGENTA), LILAC, glow))
    }

    /// How far a thing that came `since` seconds ago has faded in, over `length` seconds; whole when it
    /// stood before the view opened.
    pub fn arrived(&self, since: Option<f64>, length: f64) -> f64 {
        since.map_or(1.0, |s| ease(s / length))
    }

    /// The shares of `steps` done and working now, each step's part eased from where it stood as it
    /// changed.
    pub fn shares(&self, steps: &[Step]) -> (f64, f64) {
        if steps.is_empty() {
            return (0.0, 0.0);
        }
        let (mut done, mut working) = (0.0, 0.0);
        for step in steps {
            let seen = self.seen.step(step.id());
            let k = self.arrived(self.seen.changed(step.id(), self.now), 0.5);
            let part = |state: Option<Phase>, s: Phase| f64::from(u8::from(state == Some(s)));
            let before = seen.and_then(|c| c.before);
            let now = Some(Phase::of(step));
            done += part(now, Phase::Done) * k + part(before, Phase::Done) * (1.0 - k);
            working += part(now, Phase::Started) * k + part(before, Phase::Started) * (1.0 - k);
        }
        let all = steps.len() as f64;
        (done / all, working / all)
    }
}

/// The colour of `style`'s text.
fn rgb(style: Style) -> Color {
    style.fg.unwrap_or(Color::Reset)
}

/// The steps by state as marks with counts, `✓ 7 · ✗ 1`, leaving out a state with none, each count
/// followed by the state's word when `words`.
pub(super) fn tally(steps: &[Step], spin: &'static str, words: bool) -> Vec<Span<'static>> {
    let mut spans = vec![];
    let looks: [(Phase, &str, Style); 5] = [
        (Phase::Done, "✓", GREEN),
        (Phase::Started, spin, CYAN),
        (Phase::Open, "○", Style::new()),
        (Phase::Dropped, "✗", RED),
        (Phase::Stale, "◌", DIM),
    ];
    for (state, glyph, look) in looks {
        let count = steps.iter().filter(|s| Phase::of(s) == state).count();
        if count == 0 {
            continue;
        }
        if !spans.is_empty() {
            spans.push(paint(DIM, " · "));
        }
        spans.push(paint(look, glyph));
        let word = if words {
            format!(" {}", state.text())
        } else {
            String::new()
        };
        spans.push(paint(Style::new(), &format!(" {count}{word}")));
    }
    spans
}

#[cfg(test)]
mod tests;
