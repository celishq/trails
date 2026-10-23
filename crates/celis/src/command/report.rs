//! `celis tree --markdown`: the trails that ids name, as Markdown for a pull request.

use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::progress::{Flags, Tally};
use crate::terminal::canvas::visible;
use crate::terminal::canvas::{join, line as styled, paint, Styled, BOLD, DIM, MAGENTA, YELLOW};
use crate::terminal::frame::{Frame, RULE};
use crate::terminal::ink::Ink;
use crate::terminal::look::look;
use crate::text::clock::took;
use crate::text::line::plural;
use crate::text::line::time;
use celis_trails::records::id::short;
use celis_trails::{Mark, Rules, State, Step, Steps, Trail, Trails};
use ratatui::text::Span;
use serde_json::json;
use std::io::{IsTerminal, Read, Write};

impl Command {
    /// Prints what `markdown` gives for the ids `args` names; on a terminal with no `--json`, a card a
    /// trail, then reads one key: `m` copies the Markdown. Writes no record.
    pub(crate) fn report(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let ids = &args.words[1..];
        if args.demo && !ids.is_empty() {
            return Err(Failure::usage(
                "tree --markdown takes ids or --demo, not both",
            ));
        }
        if args.demo {
            return self.demo(args, out);
        }
        if ids.is_empty() {
            return Err(Failure::usage("tree --markdown needs an id"));
        }
        if !self.held()? {
            return Err(Failure::no_step(&ids[0]));
        }
        let trails = self.trails()?;
        let text = markdown(&trails.steps(), &trails.rules(), ids)?;
        if !std::io::stdout().is_terminal() || args.json {
            return super::show(out, args.json, "report", json!(text), text.trim_end());
        }
        let width = self
            .columns()
            .map_or(WIDTH, |c| c.saturating_sub(1).min(WIDTH));
        let mut drawn = vec![];
        for trail in chosen(&trails.steps(), ids)? {
            let card = card(&trails, trail, width);
            if !drawn.is_empty() {
                drawn.push(String::new());
            }
            let height = card.rows.len() + 2;
            let lines = card.lines(width, height);
            drawn.extend(lines.iter().map(|l| styled(l, self.colour())));
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
fn card(trails: &Trails, trail: Step, width: usize) -> Frame {
    let step = trail.record();
    let tree = &trail.tree()[1..];
    let tally = Tally::of(tree);
    let rules = trails.rules().of(trail).len();
    let ran = took(trail.newest().saturating_sub(trail.note().created_at));
    let state = trail.state();
    let mark = paint(look(state).text, look(state).mark);
    let labels = Styled::from(vec![
        paint(DIM, "STEPS   RULES   REVISED   DROPPED   "),
        mark,
        format!(" {}", state.text().to_uppercase()).into(),
    ]);
    let values = format!(
        "{:<8}{:<8}{:<10}{}",
        tree.len(),
        rules,
        tally.revised,
        tally.dropped
    );
    let mut rows = vec![Styled::from(paint(BOLD, step.title())), RULE.into()];
    match width < NARROW {
        true => rows.extend([
            Styled::from(vec!["took ".into(), paint(BOLD, &ran)]),
            labels,
            values.into(),
        ]),
        false => {
            let ink = Ink { plain: false };
            rows.extend(ink.beside(&ran, &[labels, values.into()], 3));
        }
    }
    if !tree.is_empty() {
        let mut cells: Vec<Span> = tree
            .iter()
            .map(|s| paint(look(s.state()).text, "▮"))
            .collect();
        let counts: Vec<String> = [
            (tally.done, "done"),
            (tally.revised, "revised"),
            (tally.dropped, "dropped"),
        ]
        .into_iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, s)| format!("{n} {s}"))
        .collect();
        cells.extend(["   ".into(), paint(DIM, &counts.join(" · "))]);
        rows.extend([Styled::default(), Styled::from(cells)]);
    }
    let parts = Trail::new(trail).parts();
    if !parts.is_empty() {
        rows.push(RULE.into());
        let inside = width - 4;
        for part in parts {
            let look = look(part.state());
            let time = time(part).unwrap_or_else(|| part.state().text().to_string());
            let title =
                crate::terminal::canvas::cut(part.title(), inside.saturating_sub(time.len() + 4));
            let pad = " ".repeat(inside.saturating_sub(visible(&title) + time.len() + 3));
            let mark = paint(look.text, look.mark);
            rows.push(Styled::from(vec![
                mark,
                format!(" {title}{pad} ").into(),
                paint(DIM, &time),
            ]));
        }
    }
    let lessons = tree.iter().filter(|s| Flags::of(**s).changed).count();
    let mut notes: Vec<Styled> = vec![];
    if rules > 0 {
        let rule = format!(" {}", plural(rules, "rule"));
        notes.push(Styled::from(vec![paint(MAGENTA, "§"), rule.into()]));
    }
    if lessons > 0 {
        let lesson = format!(" {}", plural(lessons, "lesson"));
        notes.push(Styled::from(vec![paint(YELLOW, "~"), lesson.into()]));
    }
    if !notes.is_empty() {
        let gap = || Styled::from("   ");
        let notes = notes.into_iter().enumerate().flat_map(|(n, note)| match n {
            0 => vec![note],
            _ => vec![gap(), note],
        });
        rows.extend([Styled::default(), join(notes)]);
    }
    Frame {
        title: format!("REPORT · {} · {}", short(trail.id()), step.anchor.name),
        rows,
        picked: false,
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
    let keys = paint(DIM, "m copy markdown · any key to leave");
    write(out, &styled(&keys.into(), colour))?;
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
        let said = crate::terminal::screen::clipboard(text, "copied markdown");
        write(
            out,
            &format!("{}\n", styled(&paint(DIM, &said).into(), colour)),
        )?;
    }
    Ok(())
}

/// The trail of the step each of `ids` names in `steps`, once, in their order, as Markdown.
pub(crate) fn markdown(steps: &Steps, rules: &Rules, ids: &[String]) -> Result<String, Failure> {
    let mut parts = Vec::new();
    for trail in chosen(steps, ids)? {
        let step = trail.record();
        let tree = &trail.tree()[1..];
        let tally = Tally::of(tree);
        let ran = took(trail.newest().saturating_sub(trail.note().created_at));
        let kept = rules.of(trail);
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
        if let Some(done) = trail.done() {
            lines.push(String::new());
            lines.push(format!("done: {}", done.content));
        }
        lines.extend(table(Trail::new(trail).parts()));
        if !kept.is_empty() {
            let count = match kept.len() {
                1 => "1 rule".to_string(),
                n => format!("{n} rules"),
            };
            let items = kept.iter().map(|(_, d)| format!("- {}", d.content));
            lines.extend(details(&count, items.collect()));
        }
        parts.push(lines.join("\n"));
    }
    Ok(parts.join("\n\n") + "\n")
}

/// The trail, step or rule `id` as Markdown: a trail as `markdown` gives it, a step as the table of it and
/// the steps under it, a rule as `- <text>`.
pub(crate) fn part(steps: &Steps, rules: &Rules, id: &str) -> Result<String, Failure> {
    if let Some((_, rule)) = rules.get(id) {
        return Ok(format!("- {}\n", rule.content));
    }
    let step = steps.find(id)?;
    if step.is_trail() {
        return markdown(steps, rules, &[id.to_string()]);
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
