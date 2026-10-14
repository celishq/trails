//! The text line of each record, as the terminal shows it.

use super::clock::took;
use celis_nostr::Note;
use celis_trails::id::short;
use celis_trails::{Decision, Grade, StepRecord};

/// `<id, first 8> [<project name>] <title>[ · <first place>]` of the step `record` that `note` holds.
pub(super) fn step(note: &Note, record: &StepRecord) -> String {
    let mut line = format!(
        "{} [{}] {}",
        short(&note.id),
        record.anchor.name,
        record.title()
    );
    if let Some(at) = record.at() {
        line.push_str(&format!(" · {at}"));
    }
    line
}

/// `<id, first 8> <scope> <text>` of the decision that `note` holds.
pub(super) fn decision(note: &Note, decision: &Decision) -> String {
    format!(
        "{} {} {}",
        short(&note.id),
        decision.scope.short(),
        decision.content
    )
}

/// `  rule <id, first 8> <text>` of the decision that `note` holds, as a rule of a trail.
pub(super) fn rule(note: &Note, decision: &Decision) -> String {
    format!("  rule {} {}", short(&note.id), decision.content)
}

/// `took <time>[ · revised by <id, first 8>, ...]`.
pub(super) fn grade(grade: &Grade) -> String {
    let by: Vec<String> = grade.revised.iter().map(|id| short(id)).collect();
    match by.is_empty() {
        true => format!("took {}", took(grade.took)),
        false => format!("took {} · revised by {}", took(grade.took), by.join(", ")),
    }
}
