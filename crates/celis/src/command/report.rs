//! `celis report`: the trails that ids name, as Markdown for a pull request.

use super::card::{paint, Card, RULE};
use super::clock::took;
use super::glyph::digits;
use super::tree::{look, visible};
use super::{Args, Command};
use celis_trails::id::short;
use celis_trails::{Failure, State, Step, Steps, Trail, Trails};
use serde_json::json;
use std::io::{IsTerminal, Read, Write};

impl Command {
    /// Prints what `markdown` gives for the ids `args` names; on a terminal with no `--json`, a card a
    /// trail, then reads one key: `m` copies the Markdown. Writes no record.
    pub(super) fn report(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let ids = &args.words[1..];
        if args.demo && !ids.is_empty() {
            return Err(Failure::usage("report takes ids or --demo, not both"));
        }
        if args.demo {
            return self.demo(args, out);
        }
        if ids.is_empty() {
            return Err(Failure::usage("report needs an id"));
        }
        if !self.held()? {
            return Err(Failure::no_step(&ids[0]));
        }
        let trails = self.trails()?;
        let text = markdown(&trails, ids)?;
        if !std::io::stdout().is_terminal() || args.json {
            return super::show(out, args.json, "report", json!(text), text.trim_end());
        }
        let width = self
            .columns()
            .map_or(WIDTH, |c| c.saturating_sub(1).min(WIDTH));
        let mut drawn = vec![];
        for trail in chosen(&trails.steps(), ids)? {
            let card = card(&trails, trail, width, self.colour());
            if !drawn.is_empty() {
                drawn.push(String::new());
            }
            let height = card.rows.len() + 2;
            drawn.extend(card.lines(width, height, self.colour()));
        }
        super::put(out, &drawn.join("\n"))?;
        leave(out, &text, self.colour())
    }
}

/// The widest a report's card prints.
const WIDTH: usize = 80;

/// The width below which a card prints its time as plain bold text.
const NARROW: usize = 60;

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

/// The card of `trail`: its id, project and title, its time and counts, its steps as a strip of their
/// states, a line a step under it, and its rules and lessons, `width` wide; its time plain when narrow.
fn card(trails: &Trails, trail: Step, width: usize, colour: bool) -> Card {
    let step = trail.record();
    let tree = &trail.tree()[1..];
    let of = |state: State| tree.iter().filter(|s| s.state() == state).count();
    let rules = trails
        .decisions()
        .rules(&step.anchor.project, trail.id())
        .len();
    let ran = took(trail.newest().saturating_sub(trail.note().created_at));
    let state = trail.state();
    let mark = paint(colour, look(state).text, look(state).mark);
    let labels = format!(
        "{}{mark} {}",
        paint(colour, "2", "STEPS   RULES   REVISED   DROPPED   "),
        state.text().to_uppercase()
    );
    let values = format!(
        "{:<8}{:<8}{:<10}{}",
        tree.len(),
        rules,
        of(State::Revised),
        of(State::Dropped)
    );
    let mut rows = vec![paint(colour, "1", step.title()), RULE.to_string()];
    match width < NARROW {
        true => rows.extend([format!("took {}", paint(colour, "1", &ran)), labels, values]),
        false => {
            let big = digits(&ran);
            let room = big.iter().map(|r| visible(r)).max().unwrap_or(0) + 3;
            let side = [labels, values, String::new()];
            for (row, side) in big.iter().zip(side) {
                let pad = " ".repeat(room - visible(row));
                rows.push(format!("{}{pad}{side}", paint(colour, "1", row)));
            }
        }
    }
    if !tree.is_empty() {
        let cells: String = tree
            .iter()
            .map(|s| paint(colour, look(s.state()).text, "▮"))
            .collect();
        let counts: Vec<String> = [State::Done, State::Revised, State::Dropped]
            .into_iter()
            .map(|s| (of(s), s.text()))
            .filter(|(n, _)| *n > 0)
            .map(|(n, s)| format!("{n} {s}"))
            .collect();
        rows.extend([
            String::new(),
            format!("{cells}   {}", paint(colour, "2", &counts.join(" · "))),
        ]);
    }
    let parts = Trail::new(trail).parts();
    if !parts.is_empty() {
        rows.push(RULE.to_string());
        let inside = width - 4;
        for part in parts {
            let look = look(part.state());
            let time = part
                .grade()
                .map_or(part.state().text().to_string(), |g| took(g.took));
            let title = super::tree::cut(part.title(), inside.saturating_sub(time.len() + 4));
            let pad = " ".repeat(inside.saturating_sub(visible(&title) + time.len() + 3));
            let mark = paint(colour, look.text, look.mark);
            rows.push(format!("{mark} {title}{pad} {}", paint(colour, "2", &time)));
        }
    }
    let lessons = tree
        .iter()
        .filter(|s| s.done().and_then(|d| d.tag("changed")).is_some())
        .count();
    let mut notes = vec![];
    if rules > 0 {
        notes.push(format!(
            "{} {}",
            paint(colour, "35", "§"),
            count(rules, "rule")
        ));
    }
    if lessons > 0 {
        notes.push(format!(
            "{} {}",
            paint(colour, "33", "~"),
            count(lessons, "lesson")
        ));
    }
    if !notes.is_empty() {
        rows.extend([String::new(), notes.join("   ")]);
    }
    Card {
        title: format!("REPORT · {} · {}", short(trail.id()), step.anchor.name),
        rows,
        picked: false,
    }
}

/// `n` of `what`, as `1 rule` or `3 rules`.
fn count(n: usize, what: &str) -> String {
    match n {
        1 => format!("1 {what}"),
        n => format!("{n} {what}s"),
    }
}

/// Prints the keys it reads and reads one key when stdin is a terminal: `m` copies `text` and says so,
/// and any other key leaves; either clears the key line.
fn leave(out: &mut dyn Write, text: &str, colour: bool) -> Result<(), Failure> {
    let write = |out: &mut dyn Write, s: &str| {
        write!(out, "{s}")
            .and_then(|_| out.flush())
            .map_err(anyhow::Error::from)
    };
    let mut before: libc::termios = unsafe { std::mem::zeroed() };
    if unsafe { libc::tcgetattr(libc::STDIN_FILENO, &mut before) } != 0 {
        return Ok(());
    }
    write(
        out,
        &paint(colour, "2", "m copy markdown · any key to leave"),
    )?;
    let mut raw = before;
    raw.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ISIG);
    raw.c_cc[libc::VMIN] = 1;
    raw.c_cc[libc::VTIME] = 0;
    unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &raw) };
    let mut key = [0u8; 1];
    let read = std::io::stdin().read(&mut key);
    unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &before) };
    write(out, "\r\x1b[2K")?;
    if read.is_ok_and(|n| n == 1) && key[0] == b'm' {
        let said = super::screen::clipboard(text, "copied markdown");
        write(out, &format!("{}\n", paint(colour, "2", &said)))?;
    }
    Ok(())
}

/// The trail of the step each of `ids` names, once, in their order, as Markdown.
pub(super) fn markdown(trails: &Trails, ids: &[String]) -> Result<String, Failure> {
    let steps = trails.steps();
    let decisions = trails.decisions();
    let mut parts = Vec::new();
    for trail in chosen(&steps, ids)? {
        let step = trail.record();
        let tree = &trail.tree()[1..];
        let of = |state: State| tree.iter().filter(|s| s.state() == state).count();
        let ran = took(trail.newest().saturating_sub(trail.note().created_at));
        let rules = decisions.rules(&step.anchor.project, trail.id());
        let mut lines = vec![
            format!("## {}", step.title()),
            String::new(),
            "| Id | State | Took | Steps | Rules | Revised | Dropped | Project |".into(),
            "|----|-------|-----:|------:|------:|--------:|--------:|---------|".into(),
            format!(
                "| `{}` | {} {} | {ran} | {} | {} | {} | {} | {} |",
                short(trail.id()),
                look(trail.state()).mark,
                trail.state().text(),
                tree.len(),
                rules.len(),
                of(State::Revised),
                of(State::Dropped),
                cell(&step.anchor.name)
            ),
        ];
        let text: Vec<&str> = step.detail().collect();
        if !text.is_empty() {
            lines.push(String::new());
            lines.extend(text.iter().map(|l| l.to_string()));
        }
        if let Some(done) = trail.done() {
            lines.push(String::new());
            lines.push(format!("done: {}", done.content));
        }
        lines.extend(table(Trail::new(trail).parts()));
        if !rules.is_empty() {
            let count = match rules.len() {
                1 => "1 rule".to_string(),
                n => format!("{n} rules"),
            };
            let items = rules.iter().map(|(_, d)| format!("- {}", d.content));
            lines.extend(details(&count, items.collect()));
        }
        parts.push(lines.join("\n"));
    }
    Ok(parts.join("\n\n") + "\n")
}

/// The trail, step or rule `id` as Markdown: a trail as `markdown` gives it, a step as the table of it and
/// the steps under it, a rule as `- <text>`.
pub(super) fn part(trails: &Trails, id: &str) -> Result<String, Failure> {
    if let Some((_, decision)) = trails.decisions().get(id) {
        return Ok(format!("- {}\n", decision.content));
    }
    let steps = trails.steps();
    let step = steps.find(id)?;
    if step.is_trail() {
        return markdown(trails, &[id.to_string()]);
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
        let time = at.grade().map_or("—".to_string(), |g| took(g.took));
        let mut proof: Vec<String> = vec![];
        if let Some(done) = at.done() {
            proof.push(done.content.clone());
            proof.extend(done.tag("changed").map(|c| format!("~ changed: {c}")));
        }
        if let Some(dropped) = at.dropped() {
            proof.push(format!("dropped: {}", dropped.content));
        }
        let title = cell(at.title());
        let proof = cell(&proof.join(" · "));
        rows.push(format!("| {mark} | {lead}{title} | {time} | {proof} |"));
        let step = at.record();
        let mut under: Vec<String> = step.detail().map(str::to_string).collect();
        under.extend(step.places.iter().map(|p| format!("at `{p}`")));
        if let Some(check) = &step.check {
            match state.reached() {
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
