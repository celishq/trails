//! The effect each event of a trail plays on the node view.

use super::events::{Event, Kind};
use ratatui::layout::Rect;
use ratatui::style::Color;
use std::collections::HashMap;
use tachyonfx::{fx, Effect, Interpolation, Motion};

/// The effect of `event` on the area it drew in `areas`: a step added comes together, a started one
/// sweeps in with its edge, a done one flashes white and a dropped one red, and a rule slides in.
pub(super) fn effect(event: &Event, areas: &HashMap<String, Rect>) -> Option<Effect> {
    let area = *areas.get(&event.id)?;
    let edge = areas.get(&format!("edge:{}", event.id)).copied();
    let timer = |ms| (ms, Interpolation::QuadOut);
    let on = |effect: Effect, at: Rect| effect.with_area(at);
    let effect = match event.kind {
        Kind::Added => on(fx::coalesce(timer(500)), area),
        Kind::Started => {
            let sweep = |at| {
                on(
                    fx::sweep_in(Motion::LeftToRight, 8, 0, Color::Black, timer(600)),
                    at,
                )
            };
            let mut both = vec![sweep(area)];
            both.extend(edge.map(sweep));
            fx::parallel(&both)
        }
        Kind::Done => {
            let flash = |at| on(fx::fade_from_fg(Color::White, timer(900)), at);
            let mut both = vec![flash(area)];
            both.extend(edge.map(flash));
            fx::parallel(&both)
        }
        Kind::Dropped => on(fx::fade_from_fg(Color::Red, timer(900)), area),
        Kind::Rule => on(
            fx::slide_in(Motion::LeftToRight, 12, 0, Color::Black, timer(700)),
            area,
        ),
    };
    Some(effect)
}
