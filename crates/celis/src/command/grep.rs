//! `celis grep`: every trail of every project as lines, through the system `grep`.

use super::clock::date;
use super::Command;
use celis_nostr::Note;
use celis_trails::id::short;
use celis_trails::{Decision, Decisions, Failure, Scope, Step, Steps, Trails};
use std::collections::HashMap;
use std::io::Write;
use std::process::Stdio;

/// The bytes the lines and their legend may take.
const LINES: usize = 14_000;
/// The bytes the whole output may take.
const OUTPUT: usize = 20_000;

impl Command {
    /// Gives `words` to the system `grep` with the view on its stdin, and prints what it prints, cut at
    /// [`LINES`], then the trails it cut, the working folder's project and the legend. Writes no record.
    pub(super) fn grep(&self, words: &[String], out: &mut dyn Write) -> Result<(), Failure> {
        let trails = match self.held()? {
            true => Some(self.trails()?),
            false => None,
        };
        let steps = trails.as_ref().map(Trails::steps);
        let decisions = trails.as_ref().map(Trails::decisions);
        let view = match (&steps, &decisions) {
            (Some(steps), Some(decisions)) => view(steps, decisions).join("\n"),
            _ => String::new(),
        };
        let found = run(words, view)?;
        if found.is_empty() {
            return super::put(out, "No line matches.");
        }
        let heads: HashMap<String, Step> = steps.as_ref().map_or_else(HashMap::new, |steps| {
            let roots = steps.roots().into_iter();
            roots.map(|s| (short(s.id()), s)).collect()
        });
        let legend = |id: &str| match heads.get(id) {
            Some(s) => format!("{} · {}", head(*s), date(s.note().created_at)),
            None => id.to_string(),
        };
        let lines: Vec<&str> = found.lines().collect();
        let (mut used, mut shown, mut seen) = (0, 0, Vec::<String>::new());
        for line in &lines {
            let new = trail(line).filter(|t| *t != "-" && !seen.iter().any(|s| s == t));
            let cost = line.len() + 1 + new.map_or(0, |t| legend(t).len() + 1);
            if shown > 0 && used + cost > LINES {
                break;
            }
            seen.extend(new.map(str::to_string));
            (used, shown) = (used + cost, shown + 1);
        }
        let mut text: Vec<String> = lines[..shown].iter().map(|l| l.to_string()).collect();
        let mut tail = vec![String::new(), format!("project [{}]", self.project().name)];
        tail.push("trails".to_string());
        tail.extend(seen.iter().map(|t| legend(t)));
        let cut = &lines[shown..];
        if !cut.is_empty() {
            let rest: usize = tail[..3].iter().map(|l| l.len() + 1).sum();
            text.extend(index(cut, &legend, OUTPUT.saturating_sub(used + rest)));
        }
        text.extend(tail);
        super::put(out, &text.join("\n"))
    }
}

/// What `grep` prints for `words` with `view` on its stdin; empty when no line matches. A `grep` that
/// fails exits 2, with its own message.
fn run(words: &[String], view: String) -> Result<String, Failure> {
    unsafe { libc::signal(libc::SIGPIPE, libc::SIG_IGN) };
    let output = feed(words, view);
    unsafe { libc::signal(libc::SIGPIPE, libc::SIG_DFL) };
    let output = output.map_err(anyhow::Error::from)?;
    match output.status.code() {
        Some(0 | 1) => Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string()),
        _ => Err(Failure {
            name: "usage".to_string(),
            message: String::new(),
            code: 2,
        }),
    }
}

/// The output of `grep` run with `words` and `view` on its stdin, which it may stop reading.
fn feed(words: &[String], view: String) -> std::io::Result<std::process::Output> {
    let mut grep = std::process::Command::new("grep")
        .args(words)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut stdin = grep.stdin.take().expect("a piped stdin");
    let feed = std::thread::spawn(move || {
        let _ = stdin.write_all(view.as_bytes());
        let _ = stdin.write_all(b"\n");
    });
    let output = grep.wait_with_output();
    let _ = feed.join();
    output
}

/// The lines of the trails cut from the output: how many, then each trail with the count of each kind
/// of its lines, while they fit in `room` bytes, then how many trails did not fit.
fn index(cut: &[&str], legend: &dyn Fn(&str) -> String, room: usize) -> Vec<String> {
    let mut counts: Vec<(&str, Vec<(&str, usize)>)> = Vec::new();
    for line in cut {
        let (Some(t), Some(kind)) = (trail(line), kind(line)) else {
            continue;
        };
        if !counts.iter().any(|(id, _)| *id == t) {
            counts.push((t, Vec::new()));
        }
        let (_, kinds) = counts.iter_mut().find(|(id, _)| *id == t).expect("a trail");
        match kinds.iter_mut().find(|(k, _)| *k == kind) {
            Some((_, n)) => *n += 1,
            None => kinds.push((kind, 1)),
        }
    }
    let trails = counts.iter().filter(|(t, _)| *t != "-").count();
    let mut text = vec![
        String::new(),
        format!(
            "celis: {} more lines in {trails} trails match; narrow the search, or open one with \
             celis tree <id> --full",
            cut.len()
        ),
    ];
    let mut used: usize = text.iter().map(|l| l.len() + 1).sum();
    for (n, (t, kinds)) in counts.iter().enumerate() {
        let tally: Vec<String> = kinds.iter().map(|(k, n)| count(*n, k)).collect();
        let line = match *t {
            "-" => format!("- · {}", tally.join(", ")),
            t => format!("{} · {}", legend(t), tally.join(", ")),
        };
        if used + line.len() + 40 > room {
            text.push(format!("… {} more trails", counts.len() - n));
            break;
        }
        used += line.len() + 1;
        text.push(line);
    }
    text
}

/// `<n> <kind>s` for a kind that is a noun, else `<n> <kind>`.
fn count(n: usize, kind: &str) -> String {
    match kind {
        "changed" | "dropped" => format!("{n} {kind}"),
        noun => super::line::plural(n, noun),
    }
}

/// The trail field of a view line that `grep` printed, after any prefix of its own.
fn trail(line: &str) -> Option<&str> {
    let rest = &line[day(line)? + 11..];
    rest.split(' ').next()
}

/// The kind field of a view line that `grep` printed.
fn kind(line: &str) -> Option<&str> {
    let rest = &line[day(line)? + 11..];
    let (_, after) = rest.split_once("] ")?;
    after.split(' ').next()
}

/// Where the first `yyyy-mm-dd ` of `line` starts.
fn day(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    (0..bytes.len().saturating_sub(10)).find(|&i| {
        let at = |k: usize| bytes[i + k];
        (0..10).all(|k| match k {
            4 | 7 => at(k) == b'-',
            _ => at(k).is_ascii_digit(),
        }) && at(10) == b' '
    })
}

/// The view: every trail of every project with its rules and steps, and each decision with no trail,
/// newest made first.
fn view<'a>(steps: &'a Steps<'a>, decisions: &Decisions<'a>) -> Vec<String> {
    let mut rules: HashMap<&str, Vec<String>> = HashMap::new();
    let mut blocks: Vec<(u64, &str, Vec<String>)> = Vec::new();
    for (note, decision, by) in decisions.every().into_iter().rev() {
        match &decision.scope {
            Scope::Trail(id) => {
                rules
                    .entry(id.as_str())
                    .or_default()
                    .push(rule(note, decision, by, &short(id)))
            }
            _ => blocks.push((
                note.created_at,
                &note.id,
                vec![rule(note, decision, by, "-")],
            )),
        }
    }
    for root in steps.roots() {
        let mut lines = vec![step(root, root, "trail")];
        lines.extend(rules.remove(root.id()).unwrap_or_default());
        level(root, root, &mut lines);
        blocks.push((root.note().created_at, root.id(), lines));
    }
    blocks.sort_by(|(a, x, _), (b, y, _)| (b, y).cmp(&(a, x)));
    blocks.into_iter().flat_map(|(.., lines)| lines).collect()
}

/// Adds the steps under `parent` of `root` to `lines`, newest made first, each with its marks and then
/// the steps under it.
fn level<'a>(root: Step<'a>, parent: Step<'a>, lines: &mut Vec<String>) {
    let mut children = parent.children();
    children.sort_by(|a, b| (b.note().created_at, b.id()).cmp(&(a.note().created_at, a.id())));
    for child in children {
        lines.push(step(root, child, "step"));
        let line = |note: &Note, kind: &str, text: &str| {
            format!(
                "{} {kind} {} {}",
                lead(root, note),
                short(child.id()),
                flat(text)
            )
        };
        if let Some(done) = child.done() {
            lines.push(line(done, "proof", &done.content));
            if let Some(changed) = done.tag("changed") {
                lines.push(line(done, "changed", changed));
            }
        }
        if let Some(dropped) = child.dropped() {
            lines.push(line(dropped, "dropped", &dropped.content));
        }
        level(root, child, lines);
    }
}

/// The line of the step `step` of the trail `root`, as `kind`.
fn step(root: Step, step: Step, kind: &str) -> String {
    let record = step.record();
    let mut fields = vec![step.state().text().to_string()];
    let by: Vec<String> = step.revised().iter().map(|s| short(s.id())).collect();
    if !by.is_empty() {
        fields.push(format!("revised-by {}", by.join(",")));
    }
    fields.extend(record.places.iter().map(|p| format!("at:{p}")));
    format!(
        "{} {kind} {} {} {}",
        lead(root, step.note()),
        short(step.id()),
        fields.join(" "),
        flat(&record.content)
    )
}

/// The line of the decision `note` holds as a rule of the trail `trail`, replaced by `by` when given.
fn rule(note: &Note, decision: &Decision, by: Option<&Note>, trail: &str) -> String {
    let state = by.map_or("live".to_string(), |b| {
        format!("replaced-by {}", short(&b.id))
    });
    format!(
        "{} {trail} [{}] rule {} {state} {} {}",
        date(note.created_at),
        decision.anchor.name,
        short(&note.id),
        decision.scope.short(),
        flat(&decision.content)
    )
}

/// `<day of note> <trail id, first 8> [<project name>]` of a line of the trail `root`.
fn lead(root: Step, note: &Note) -> String {
    let name = &root.record().anchor.name;
    format!("{} {} [{name}]", date(note.created_at), short(root.id()))
}

/// `<id, first 8> [<project name>] <title>` of the trail `trail`.
fn head(trail: Step) -> String {
    let record = trail.record();
    format!(
        "{} [{}] {}",
        short(trail.id()),
        record.anchor.name,
        record.title()
    )
}

/// `text` on one line.
fn flat(text: &str) -> String {
    text.split(['\n', '\r'])
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
