//! The text line of each record, as the terminal shows it.

use super::clock::took;
use crate::progress::Grade;
use celis_nostr::Note;
use celis_trails::records::id::short;
use celis_trails::{Example, Pattern, ProofFile, Step, Trails};

/// `<id, first 8> note <first line>` of the note `note`, then each next line of its text.
pub(crate) fn note(note: &Note) -> String {
    let mut text = note.content.lines();
    let head = format!("{} note {}", short(&note.id), text.next().unwrap_or(""));
    let lines = std::iter::once(head).chain(text.map(|l| format!("  {l}")));
    lines.collect::<Vec<_>>().join("\n")
}

/// `<id, first 8> <scope> <first line>` of `pattern`, then each next line of its
/// text, then each of its examples: a step with its proof and blobs, a file with its blob, or a URL.
pub(crate) fn pattern(pattern: &Pattern, trails: &Trails) -> String {
    let (note, pattern) = (pattern.note, &pattern.record);
    let steps = trails.steps();
    let mut text = pattern.content.lines();
    let head = format!(
        "{} {} {}",
        short(&note.id),
        pattern.scope.text(),
        text.next().unwrap_or("")
    );
    let mut lines = vec![head];
    lines.extend(text.map(|l| format!("  {l}")));
    for (side, example) in pattern.examples.sides() {
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
