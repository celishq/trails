//! What `grep` prints of the lines it found.

use crate::command::grep::view::header::{self, Header};
use crate::command::grep::view::row::Row;
use crate::command::grep::view::View;

/// The lines found of one trail, `None` for those of no trail, each with what `grep` put before it.
struct Group<'a> {
    trail: Option<String>,
    lines: Vec<(&'a str, &'a Row)>,
}

/// The lines `grep` found in the view, grouped by trail, most lines first.
pub(super) struct Found<'a> {
    view: &'a View,
    groups: Vec<Group<'a>>,
}

impl<'a> Found<'a> {
    /// The lines of `found` that are lines of `view`; none when no line is.
    pub fn of(found: &'a str, view: &'a View) -> Option<Found<'a>> {
        let mut groups: Vec<Group> = Vec::new();
        for line in found.lines() {
            let Some(found) = view.row(line) else {
                continue;
            };
            let row = found.1;
            match groups.iter_mut().find(|g| g.trail == row.trail) {
                Some(group) => group.lines.push(found),
                None => groups.push(Group {
                    trail: row.trail.clone(),
                    lines: vec![found],
                }),
            }
        }
        if groups.is_empty() {
            return None;
        }
        groups.sort_by_key(|g| std::cmp::Reverse(g.lines.len()));
        Some(Found { view, groups })
    }

    /// Each trail's header and its lines, then the project of `project`.
    pub fn text(&self, project: &str) -> String {
        let mut text = Vec::new();
        for group in &self.groups {
            text.push(header(self.view, &group.trail));
            let lines = group.lines.iter();
            text.extend(lines.map(|(before, row)| format!("  {before}{}", row.shown())));
        }
        text.extend([String::new(), format!("project [{project}]")]);
        text.join("\n")
    }
}

/// The header of the lines of `trail`.
fn header(view: &View, trail: &Option<String>) -> String {
    trail
        .as_deref()
        .and_then(|t| view.header(t))
        .map_or(header::NONE.to_string(), Header::text)
}
