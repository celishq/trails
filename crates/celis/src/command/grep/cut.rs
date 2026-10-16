//! What `grep` prints of the lines it found.

use super::kind::Kind;
use super::view::{Row, View};

/// The bytes the lines and their headers may take.
const LINES: usize = 6_500;
/// The bytes the whole output may take.
const OUTPUT: usize = 8_000;
/// The bytes kept for the line that says how many trails did not fit.
const MORE: usize = 40;
/// The characters of a title the index of the trails cut keeps.
const TITLE: usize = 80;

/// The lines found of one trail, `None` for those of no trail, each with what `grep` put before it.
struct Group<'a> {
    trail: Option<String>,
    lines: Vec<(&'a str, &'a Row)>,
}

/// The lines found, grouped by trail, most lines first, split where they and their headers pass
/// [`LINES`] bytes.
pub(super) struct Cut<'a> {
    shown: Vec<Group<'a>>,
    rest: Vec<Group<'a>>,
}

impl<'a> Cut<'a> {
    pub fn of(found: &'a str, view: &'a View) -> Cut<'a> {
        let mut groups: Vec<Group> = Vec::new();
        for line in found.lines() {
            let Some(row) = view.row(line) else { continue };
            let found = (&line[..line.len() - row.line.len()], row);
            match groups.iter_mut().find(|g| g.trail == row.trail) {
                Some(group) => group.lines.push(found),
                None => groups.push(Group {
                    trail: row.trail.clone(),
                    lines: vec![found],
                }),
            }
        }
        groups.sort_by_key(|g| std::cmp::Reverse(g.lines.len()));
        let (mut used, mut shown, mut rest) = (0, Vec::new(), Vec::new());
        for mut group in groups {
            if !rest.is_empty() {
                rest.push(group);
                continue;
            }
            used += header(view, &group.trail).len() + 1;
            let fit = group.lines.iter().take_while(|(before, row)| {
                used += before.len() + row.body().len() + 3;
                used <= LINES
            });
            let fit = fit.count().max(usize::from(shown.is_empty()));
            if fit < group.lines.len() {
                let lines = group.lines.split_off(fit);
                rest.push(Group {
                    trail: group.trail.clone(),
                    lines,
                });
            }
            if fit > 0 {
                shown.push(group);
            }
        }
        Cut { shown, rest }
    }

    /// Each trail's header and its lines; then, when lines were cut, the index of their trails; then the
    /// project of `project`, within [`OUTPUT`] bytes. What `grep` printed, when no line is of the view.
    pub fn text(&self, view: &View, project: &str, found: &str) -> String {
        if self.shown.is_empty() {
            return found.to_string();
        }
        let mut text = Vec::new();
        for group in &self.shown {
            text.push(header(view, &group.trail));
            let lines = group.lines.iter();
            text.extend(lines.map(|(before, row)| format!("  {before}{}", row.body())));
        }
        let end = [String::new(), format!("project [{project}]")];
        if !self.rest.is_empty() {
            let used = bytes(&text) + bytes(&end);
            text.extend(self.index(view, OUTPUT.saturating_sub(used)));
        }
        text.extend(end);
        text.join("\n")
    }

    /// How many lines were cut, then each trail they belong to with the count of each kind, while they
    /// fit in `room` bytes, then how many trails did not fit.
    fn index(&self, view: &View, room: usize) -> Vec<String> {
        let lines: usize = self.rest.iter().map(|g| g.lines.len()).sum();
        let trails = self.rest.iter().filter(|g| g.trail.is_some()).count();
        let mut text = vec![
            String::new(),
            format!(
                "celis: {lines} more lines in {trails} trails match; narrow the search, or search \
                 one trail with celis grep \" <id> \\[\""
            ),
        ];
        let mut used = bytes(&text);
        for (n, group) in self.rest.iter().enumerate() {
            let mut kinds = [0; 6];
            for (_, row) in &group.lines {
                kinds[row.kind as usize] += 1;
            }
            let tally: Vec<String> = Kind::ALL
                .iter()
                .zip(kinds)
                .filter(|(_, n)| *n > 0)
                .map(|(k, n)| k.count(n))
                .collect();
            let head = group
                .trail
                .as_deref()
                .map_or("-".to_string(), |t| view.short(t, TITLE));
            let line = format!("{head} · {}", tally.join(", "));
            if used + line.len() + MORE > room {
                text.push(format!("… {} more trails", self.rest.len() - n));
                break;
            }
            used += line.len() + 1;
            text.push(line);
        }
        text
    }
}

/// The header of the lines of `trail`: its title, or `- rules of no trail`.
fn header(view: &View, trail: &Option<String>) -> String {
    trail
        .as_deref()
        .map_or("- rules of no trail".to_string(), |t| view.title(t))
}

/// The bytes `lines` take, each with its newline.
fn bytes(lines: &[String]) -> usize {
    lines.iter().map(|l| l.len() + 1).sum()
}
