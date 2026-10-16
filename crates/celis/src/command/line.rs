//! The text line of each record, as the terminal shows it.

use super::clock::took;
use celis_nostr::Note;
use celis_trails::id::short;
use celis_trails::{Decision, Step};

/// `<id, first 8> <scope> <text>` of the decision that `note` holds.
pub(super) fn decision(note: &Note, decision: &Decision) -> String {
    format!(
        "{} {} {}",
        short(&note.id),
        decision.scope.short(),
        decision.content
    )
}

/// `took <time>[ · revised by <id, first 8>, ...]` of a done `step`; `None` while it is not done.
pub(super) fn grade(step: Step) -> Option<String> {
    let took = took(step.grade()?.took);
    let by: Vec<String> = step.revised().iter().map(|s| short(s.id())).collect();
    Some(match by.is_empty() {
        true => format!("took {took}"),
        false => format!("took {took} · revised by {}", by.join(", ")),
    })
}

/// `<n> <word>s`, or `1 <word>`.
pub(super) fn plural(n: usize, word: &str) -> String {
    match n {
        1 => format!("1 {word}"),
        n => format!("{n} {word}s"),
    }
}
