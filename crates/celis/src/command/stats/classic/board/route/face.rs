//! The step of a route as its card: its time big, and how often it was revised.

use crate::progress::Phase;
use crate::terminal::canvas::Styled;
use crate::terminal::classic::look;
use crate::terminal::frame::Frame;
use crate::terminal::ink::Ink;
use crate::text::line::time;
use celis_trails::Step;

/// The lines of the card of `step`, `width` wide and framed as picked; its time plain when `plain`.
pub(super) fn face(step: Step, width: usize, plain: bool) -> Vec<Styled> {
    let ink = Ink { plain };
    let took = time(step).unwrap_or("—".into());
    let side = [
        ink.dim("took").into(),
        ink.dim(&format!("revised {}", step.revised().len())).into(),
    ];
    let look = look(Phase::of(&step));
    let card = Frame {
        title: format!("{} {}", look.mark, step.title()),
        rows: ink.beside(&took, &side, 3),
        picked: true,
    };
    card.lines(width, card.rows.len() + 2)
}
