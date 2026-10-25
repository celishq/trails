//! The text line of each record, as the terminal shows it.

use super::clock::took;
use crate::progress::Grade;
use celis_nostr::Note;
use celis_trails::records::id::short;
use celis_trails::{Example, ProofFile, Rule, Step, Trails};

/// `<id, first 8> <scope> <text>` of the rule that `note` holds, then each of its examples: a step
/// with its proof and blobs, a file with its blob, or a URL.
pub(crate) fn rule(note: &Note, rule: &Rule, trails: &Trails) -> String {
    let steps = trails.steps();
    let head = format!(
        "{} {} {}",
        short(&note.id),
        rule.scope.short(),
        rule.content
    );
    let mut lines = vec![head];
    for (side, example) in rule.examples.sides() {
        let id = match example {
            Example::Step(id) => id,
            Example::File(file) => {
                lines.push(format!("  {side:<4}    {}", file.place));
                lines.push(format!("          {}", blob(file, trails)));
                continue;
            }
            Example::Url(url) => {
                lines.push(format!("  {side:<4}    {url}"));
                continue;
            }
        };
        let Ok(step) = steps.find(id) else {
            continue;
        };
        let done = step.done();
        let proof = done.as_ref().and_then(|d| d.proof.lines().next());
        let head = format!("{} {}", short(step.id()), step.title());
        lines.push(format!("  {side:<4}    {head} · {}", proof.unwrap_or("")));
        let files = done.iter().flat_map(|d| d.files.iter());
        lines.extend(files.map(|f| format!("          {}", blob(f, trails))));
    }
    lines.join("\n")
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

/// Where `trails` keeps the blob of `file`; its place and sha256 when the store lacks it.
pub(crate) fn blob(file: &ProofFile, trails: &Trails) -> String {
    match trails.blob(&file.sha256) {
        Some(path) => path.display().to_string(),
        None => format!("{} · {} missing", file.place, file.sha256),
    }
}
