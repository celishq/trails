//! The key line of a live screen: each key the person presses bold, what it does plain, all dim.

use super::canvas::{paint, Styled, BOLD, DIM};

/// The words of a key line that name a key.
const KEYS: [&str; 23] = [
    "↑↓",
    "←→",
    "←↑↓→",
    "↑",
    "↓",
    "←",
    "→",
    "enter",
    "esc",
    "tab",
    "space",
    "PgUp",
    "PgDn",
    "/",
    "*",
    "a",
    "c",
    "f",
    "m",
    "q",
    "r",
    "[",
    "]",
];

/// `text`, parts split by ` · `, as a dim line with the keys that lead each part bold, and an `or`
/// between two keys plain.
pub(crate) fn legend(text: &str) -> Styled {
    let bold = DIM.patch(BOLD);
    let mut spans = vec![];
    for (n, part) in text.split(" · ").enumerate() {
        if n > 0 {
            spans.push(paint(DIM, " · "));
        }
        let words: Vec<&str> = part.split(' ').collect();
        let key = |k: usize| words.get(k).is_some_and(|w| KEYS.contains(w));
        let mut k = 0;
        while key(k) || (words[k] == "or" && k > 0 && key(k + 1)) {
            let gap = if k > 0 { " " } else { "" };
            match words[k] {
                "or" => spans.push(paint(DIM, &format!("{gap}or"))),
                word => spans.push(paint(bold, &format!("{gap}{word}"))),
            }
            k += 1;
            if k == words.len() {
                break;
            }
        }
        let rest = words[k..].join(" ");
        if !rest.is_empty() {
            let gap = if k > 0 { " " } else { "" };
            spans.push(paint(DIM, &format!("{gap}{rest}")));
        }
    }
    Styled::from(spans)
}

#[cfg(test)]
mod tests;
