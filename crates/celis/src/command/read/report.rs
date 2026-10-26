//! The trails that ids name, as Markdown, for `celis issue` and the copy key of the watch.

use crate::failure::Failure;
use crate::progress::Tally;
use crate::terminal::look::look;
use crate::text::clock::took;
use crate::text::line::time;
use celis_trails::records::id::short;
use celis_trails::{Mark, Notes, Patterns, State, Step, Steps, Trail};

/// The trail of the step each of `ids` names, once, in their order.
fn chosen<'a>(steps: &'a Steps<'a>, ids: &[String]) -> Result<Vec<Step<'a>>, Failure> {
    let mut chosen: Vec<Step> = Vec::new();
    for id in ids {
        let trail = steps.find(id)?.trail();
        if !chosen.iter().any(|s| s.id() == trail.id()) {
            chosen.push(trail);
        }
    }
    Ok(chosen)
}

/// The trail of the step each of `ids` names in `steps`, once, in their order, as Markdown.
pub(crate) fn markdown(steps: &Steps, notes: &Notes, ids: &[String]) -> Result<String, Failure> {
    let mut parts = Vec::new();
    for trail in chosen(steps, ids)? {
        let step = trail.record();
        let tree = &trail.tree()[1..];
        let tally = Tally::of(tree);
        let ran = took(trail.newest().saturating_sub(trail.note().created_at));
        let kept = notes.of(trail);
        let mut lines = vec![
            format!("## {}", step.title()),
            String::new(),
            "| Id | State | Took | Steps | Notes | Revised | Dropped | Project |".into(),
            "|----|-------|-----:|------:|------:|--------:|--------:|---------|".into(),
            format!(
                "| `{}` | {} {} | {ran} | {} | {} | {} | {} | {} |",
                short(trail.id()),
                look(trail.state()).mark,
                trail.state().text(),
                tree.len(),
                kept.len(),
                tally.revised,
                tally.dropped,
                cell(&step.anchor.name)
            ),
        ];
        let text: Vec<&str> = step.detail().collect();
        if !text.is_empty() {
            lines.push(String::new());
            lines.extend(text.iter().map(|l| l.to_string()));
        }
        if let Some(done) = trail.done_record() {
            lines.push(String::new());
            lines.push(format!("done: {}", done.content));
        }
        lines.extend(table(Trail::new(trail).parts()));
        if !kept.is_empty() {
            let count = match kept.len() {
                1 => "1 note".to_string(),
                n => format!("{n} notes"),
            };
            let items = kept.iter().map(|n| format!("- {}", n.content));
            lines.extend(details(&count, items.collect()));
        }
        parts.push(lines.join("\n"));
    }
    Ok(parts.join("\n\n") + "\n")
}

/// The trail, step, pattern or note `id` as Markdown: a trail as `markdown` gives it, a step as the table
/// of it and the steps under it, a pattern or a note as `- <text>`.
pub(crate) fn part(
    steps: &Steps,
    patterns: &Patterns,
    notes: &Notes,
    id: &str,
) -> Result<String, Failure> {
    if let Ok(pattern) = patterns.find(None, id) {
        return Ok(format!("- {}\n", pattern.record.content));
    }
    if let Some(note) = notes.get(id) {
        return Ok(format!("- {}\n", note.content));
    }
    let step = steps.find(id)?;
    if step.is_trail() {
        return markdown(steps, notes, &[id.to_string()]);
    }
    let lines = table(vec![step]);
    Ok(lines[1..].join("\n") + "\n")
}

/// A blank line, then the table of `steps` and every step under them, with their marks, times and
/// proofs, then their texts, places and checks in a details block; nothing with no steps.
fn table(steps: Vec<Step>) -> Vec<String> {
    if steps.is_empty() {
        return vec![];
    }
    let mut rows = vec![];
    let mut notes = vec![];
    for (depth, at) in deep(steps, 0) {
        let state = at.state();
        let mark = look(state).mark;
        let lead = match depth {
            0 => String::new(),
            n => format!("{}└ ", "\u{a0}\u{a0}".repeat(n)),
        };
        let time = time(at).unwrap_or("—".to_string());
        let proof: Vec<String> = match at.close() {
            Some((_, Mark::Done(celis_trails::Done { proof, changed, .. }))) => {
                let changed = changed.map(|c| format!("~ changed: {c}"));
                std::iter::once(proof).chain(changed).collect()
            }
            Some((_, Mark::Drop { reason })) => vec![format!("dropped: {reason}")],
            _ => vec![],
        };
        let title = cell(at.title());
        let proof = cell(&proof.join(" · "));
        rows.push(format!("| {mark} | {lead}{title} | {time} | {proof} |"));
        let step = at.record();
        let mut under: Vec<String> = step.detail().map(str::to_string).collect();
        under.extend(step.places.iter().map(|p| format!("at `{p}`")));
        if let Some(check) = &step.check {
            match state == State::Done {
                true => under.push(format!("check passed `{check}`")),
                false => under.push(format!("check `{check}`")),
            }
        }
        if !under.is_empty() {
            notes.push(format!("- {mark} {}", at.title()));
            notes.extend(under.iter().map(|l| format!("  - {l}")));
        }
    }
    let mut lines = vec![
        String::new(),
        "| | Step | Took | Proof |".into(),
        "|-|------|-----:|-------|".into(),
    ];
    lines.extend(rows);
    if !notes.is_empty() {
        lines.extend(details("Texts, places and checks", notes));
    }
    lines
}

/// Each step of `steps` at `depth`, each followed by the steps under it, one level deeper.
fn deep(steps: Vec<Step>, depth: usize) -> Vec<(usize, Step)> {
    let mut all = vec![];
    for step in steps {
        all.push((depth, step));
        all.extend(deep(step.children(), depth + 1));
    }
    all
}

/// A blank line, then `lines` in a details block that `summary` names.
fn details(summary: &str, lines: Vec<String>) -> Vec<String> {
    let mut block = vec![
        String::new(),
        format!("<details><summary>{summary}</summary>"),
        String::new(),
    ];
    block.extend(lines);
    block.extend([String::new(), "</details>".into()]);
    block
}

/// `text` as one table cell: on one line, with each `|` escaped.
fn cell(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('|', "\\|")
}
