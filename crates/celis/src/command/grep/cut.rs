//! What `grep` prints of the lines it found.

use super::kind::Kind;
use super::view::View;

/// The bytes the lines and their legend may take.
const LINES: usize = 14_000;
/// The bytes the whole output may take.
const OUTPUT: usize = 20_000;
/// The bytes kept for the line that says how many trails did not fit.
const MORE: usize = 40;

/// The lines found, split where they and the legend of their trails pass [`LINES`] bytes.
pub(super) struct Cut<'a> {
    shown: Vec<&'a str>,
    trails: Vec<String>,
    rest: Vec<&'a str>,
}

impl<'a> Cut<'a> {
    pub fn of(found: &'a str, view: &View) -> Cut<'a> {
        let lines: Vec<&str> = found.lines().collect();
        let (mut used, mut shown, mut trails) = (0, 0, Vec::<String>::new());
        for line in &lines {
            let trail = view.row(line).and_then(|r| r.trail.clone());
            let new = trail.filter(|t| !trails.contains(t));
            let cost = line.len() + 1 + new.as_ref().map_or(0, |t| view.title(t).len() + 1);
            if shown > 0 && used + cost > LINES {
                break;
            }
            trails.extend(new);
            (used, shown) = (used + cost, shown + 1);
        }
        let (shown, rest) = lines.split_at(shown);
        Cut {
            shown: shown.to_vec(),
            trails,
            rest: rest.to_vec(),
        }
    }

    /// The lines that fit; then, when lines were cut, the index of their trails; then the project of
    /// `project` and the legend, within [`OUTPUT`] bytes.
    pub fn text(&self, view: &View, project: &str) -> String {
        let mut text: Vec<String> = self.shown.iter().map(|l| l.to_string()).collect();
        let mut legend = vec![
            String::new(),
            format!("project [{project}]"),
            "trails".to_string(),
        ];
        legend.extend(self.trails.iter().map(|t| view.title(t)));
        if !self.rest.is_empty() {
            let used = bytes(&text) + bytes(&legend);
            text.extend(self.index(view, OUTPUT.saturating_sub(used)));
        }
        text.extend(legend);
        text.join("\n")
    }

    /// How many lines were cut, then each trail they belong to with the count of each kind, while they
    /// fit in `room` bytes, then how many trails did not fit.
    fn index(&self, view: &View, room: usize) -> Vec<String> {
        let mut counts: Vec<(Option<String>, [usize; 6])> = Vec::new();
        for row in self.rest.iter().filter_map(|l| view.row(l)) {
            let at = match counts.iter().position(|(t, _)| *t == row.trail) {
                Some(at) => at,
                None => {
                    counts.push((row.trail.clone(), [0; 6]));
                    counts.len() - 1
                }
            };
            counts[at].1[row.kind as usize] += 1;
        }
        let trails = counts.iter().filter(|(t, _)| t.is_some()).count();
        let mut text = vec![
            String::new(),
            format!(
                "celis: {} more lines in {trails} trails match; narrow the search, or open one with \
                 celis tree <id> --full",
                self.rest.len()
            ),
        ];
        let mut used = bytes(&text);
        for (n, (trail, kinds)) in counts.iter().enumerate() {
            let tally: Vec<String> = Kind::ALL
                .iter()
                .zip(kinds)
                .filter(|(_, n)| **n > 0)
                .map(|(k, n)| k.count(*n))
                .collect();
            let head = trail.as_deref().map_or("-".to_string(), |t| view.title(t));
            let line = format!("{head} · {}", tally.join(", "));
            if used + line.len() + MORE > room {
                text.push(format!("… {} more trails", counts.len() - n));
                break;
            }
            used += line.len() + 1;
            text.push(line);
        }
        text
    }
}

/// The bytes `lines` take, each with its newline.
fn bytes(lines: &[String]) -> usize {
    lines.iter().map(|l| l.len() + 1).sum()
}
