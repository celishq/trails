//! The value card: what shows that celis pays, and the lifecycle of the patterns behind it.

use super::super::super::lifecycle::Lifecycle;
use super::super::super::value::Value;
use super::super::table::{Row, Table};
use super::Card;
use crate::progress::Model;
use crate::terminal::canvas::Styled;
use crate::terminal::ink::Ink;
use crate::text::clock::date;

/// The value of the last 28 days: the asks that used a pattern, the corrections and the repeat ones,
/// first-pass success with and without a pattern, and the recall funnel; it opens to the lifecycle.
pub(super) fn value(model: &Model, ink: Ink) -> Card {
    let value = Value::of(model, None);
    let percent = |p: Option<u64>| p.map_or("-".to_string(), |p| format!("{p}%"));
    let face = match value.asks {
        0 => vec![ink.dim("no ask done in 28 days").into()],
        _ => vec![
            Styled::raw(format!(
                "{} of {} asks used a pattern",
                value.covered, value.asks
            )),
            Styled::raw(format!(
                "{} corrections · {} failed · {} missed",
                value.corrections, value.failed, value.missed
            )),
            Styled::raw(format!(
                "first pass {} with · {} without",
                percent(value.with),
                percent(value.without)
            )),
            Styled::raw(format!(
                "{} recalled → {} used → {} held",
                value.recalled, value.used, value.held
            )),
        ],
    };
    let rows = Lifecycle::of(model, None)
        .into_iter()
        .map(|s| Row {
            id: None,
            cells: vec![
                format!("{} {}", s.span, date(s.from)),
                s.written.to_string(),
                s.replaced.to_string(),
                s.dropped.to_string(),
                s.promoted.to_string(),
                s.recalled.to_string(),
                s.used.to_string(),
                s.misled.to_string(),
            ],
        })
        .collect();
    Card {
        title: "06 VALUE · 28 DAYS".into(),
        face,
        table: Table {
            head: vec![
                "Span", "Written", "Replaced", "Dropped", "Promoted", "Recalled", "Used", "Misled",
            ],
            rows,
            wide: 0,
        },
    }
}
