//! The text line of each record, as the terminal shows it.

use super::clock::took;
use crate::progress::Grade;
use celis_nostr::Note;
use celis_trails::records::id::short;
use celis_trails::{Decision, Step};

/// `<id, first 8> <scope> <text>` of the decision that `note` holds.
pub(crate) fn decision(note: &Note, decision: &Decision) -> String {
    format!(
        "{} {} {}",
        short(&note.id),
        decision.scope.short(),
        decision.content
    )
}

/// How long a done `step` took, as `46m 3s`; `None` while it is not done.
pub(crate) fn time(step: Step) -> Option<String> {
    Grade::of(step).map(|g| took(g.took))
}

/// `took <time>[ · revised by <id, first 8>, ...]` of a done `step`; `None` while it is not done.
pub(crate) fn grade(step: Step) -> Option<String> {
    let took = time(step)?;
    let by: Vec<String> = step.revised().iter().map(|s| short(s.id())).collect();
    Some(match by.is_empty() {
        true => format!("took {took}"),
        false => format!("took {took} · revised by {}", by.join(", ")),
    })
}

/// `<n> <word>s`, or `1 <word>`.
pub(crate) fn plural(n: usize, word: &str) -> String {
    match n {
        1 => format!("1 {word}"),
        n => format!("{n} {word}s"),
    }
}
