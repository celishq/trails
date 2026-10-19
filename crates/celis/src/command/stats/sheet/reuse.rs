//! The reuse cards: the steps followed most, and the rules later steps used.

use super::super::table::{Row, Table};
use super::Sheet;
use crate::progress::Reuse;
use crate::terminal::canvas::{cut as text_cut, visible, Styled};
use crate::terminal::glyph::bar;
use crate::terminal::ink::Ink;
use celis_trails::Rules;

/// The reuse: the follows, and the steps followed most.
pub(super) fn followed(reuse: &Reuse, inside: usize, ink: Ink) -> Sheet {
    let side = [Styled::from(ink.dim("follows"))];
    let mut face = ink.beside(&reuse.total().to_string(), &side, 2);
    face.push(Styled::default());
    let steps = reuse.followed();
    if steps.is_empty() {
        face.push(ink.dim("no step followed another yet").into());
    }
    for &step in steps.iter().take(3) {
        let tail = format!(" ──▶ {:>3}", reuse.followers(step));
        let title = text_cut(step.title(), inside.saturating_sub(visible(&tail) + 1));
        let pad = " ".repeat(inside.saturating_sub(visible(&title) + visible(&tail)));
        face.push(format!("{title}{pad}{tail}").into());
    }
    let rows = steps
        .iter()
        .map(|&s| Row {
            id: Some(s.id().to_string()),
            cells: vec![
                s.title().to_string(),
                s.record().anchor.name.clone(),
                reuse.followers(s).to_string(),
            ],
        })
        .collect();
    Sheet {
        title: "01 REUSE".into(),
        face,
        table: Table {
            head: vec!["Step", "Project", "Followed"],
            rows,
            wide: 0,
        },
    }
}

/// The rules used: the rules later steps used, most used first, as bars.
pub(super) fn rules(rules: &Rules, reuse: &Reuse, inside: usize, ink: Ink) -> Sheet {
    let used = reuse.used(rules);
    let most = used.first().map_or(0, |(_, _, n)| *n);
    let label = 18.min(inside / 2);
    let width = inside.saturating_sub(label + 5);
    let mut face: Vec<Styled> = used
        .iter()
        .take(5)
        .map(|(_, d, n)| {
            let name = text_cut(d.brief().trim_start_matches("When "), label);
            let pad = " ".repeat(label - visible(&name));
            format!("{name}{pad} {} {n:>2}", bar(*n, most, width)).into()
        })
        .collect();
    let total: usize = used.iter().map(|(_, _, n)| n).sum();
    match used.len() {
        0 => face.push(ink.dim("no rule used yet").into()),
        n => {
            let more = match n.saturating_sub(5) {
                0 => String::new(),
                k => format!("+ {k} more rules"),
            };
            let uses = format!("{total} uses");
            let pad = " ".repeat(inside.saturating_sub(visible(&more) + uses.len()));
            face.extend([
                Styled::default(),
                ink.dim(&format!("{more}{pad}{uses}")).into(),
            ]);
        }
    }
    let rows = used
        .iter()
        .map(|(note, d, n)| Row {
            id: Some(note.id.clone()),
            cells: vec![d.content.clone(), n.to_string()],
        })
        .collect();
    Sheet {
        title: "02 RULES USED".into(),
        face,
        table: Table {
            head: vec!["Rule", "Used"],
            rows,
            wide: 0,
        },
    }
}
