//! `celis search`: the rules, the other patterns and the past asks that fit an ask.

#[cfg(feature = "search")]
mod meaning;
#[cfg(not(feature = "search"))]
#[path = "search/off.rs"]
mod meaning;
pub(crate) mod recalls;
#[cfg(feature = "search")]
mod setup;

use super::view::kind::Kind;
use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::text::clock::date;
use recalls::Recalls;
use celis_nostr::Note;
use celis_trails::records::id::short;
use celis_trails::{Pattern, Step, Trails};
use std::collections::{HashMap, HashSet};
use std::io::Write;

impl Command {
    /// Prints the live rules in scope that fit the ask of `args`, by its files, its words or its meaning,
    /// then the other patterns that fit it, each with its examples, then the done asks like it with
    /// their steps, and marks each pattern as recalled now; with `--kind`, only that part. Writes no
    /// record.
    pub(crate) fn search(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> Result<(), Failure> {
        let ask = args.required_text("search needs the ask after --")?;
        let limit = Args::number("--limit", &args.limit)?;
        let parts = match args.kind.as_deref() {
            None => vec![Part::Rules, Part::Patterns, Part::Past],
            Some("rule") => vec![Part::Rules],
            Some("pattern") => vec![Part::Patterns],
            Some("trail") => {
                return self.listed(
                    &ask,
                    Kind::Trail,
                    (limit.unwrap_or(20), args.json),
                    out,
                    err,
                )
            }
            Some("step") => {
                return self.listed(&ask, Kind::Step, (limit.unwrap_or(20), args.json), out, err)
            }
            Some(kind) => {
                return Err(Failure::usage(format!(
                    "--kind takes rule, pattern, trail or step, not {kind}"
                )))
            }
        };
        if args.json {
            return Err(Failure::usage("--json takes --kind trail or --kind step"));
        }
        if !self.held()? {
            return super::put(out, NOTHING);
        }
        let limit = limit.unwrap_or(usize::MAX);
        let trails = self.trails()?;
        let found = self.fits(&trails, &ask, &args.words[1..], err)?;
        let steps = trails.steps();
        let past = match parts.contains(&Part::Past) {
            true => self.past(&ask, &steps.all(), err)?,
            false => vec![],
        };
        let mut said = vec![];
        for part in parts {
            let lines: Vec<String> = match part {
                Part::Past => past
                    .iter()
                    .take(limit.min(PAST))
                    .flat_map(|a| lines(*a))
                    .collect(),
                _ => {
                    let part: Vec<_> = found
                        .iter()
                        .filter(|(_, pattern)| pattern.binds == (part == Part::Rules))
                        .take(limit)
                        .collect();
                    if let Ok(home) = self.home() {
                        Recalls::mark(&home, part.iter().map(|(note, _)| *note));
                    }
                    let project = self.project().anchor.project;
                    let line = |(note, pattern): &&(&Note, Pattern)| {
                        let mut lines = vec![crate::text::line::pattern(note, pattern, &trails)];
                        let smaller = trails.patterns().smaller(&project, &note.id);
                        lines.extend(smaller.iter().take(NEXT).map(|(note, pattern)| {
                            let first = pattern.content.lines().next().unwrap_or("");
                            format!("  next    {} {first}", short(&note.id))
                        }));
                        lines.join("\n")
                    };
                    part.iter().map(line).collect()
                }
            };
            if lines.is_empty() {
                continue;
            }
            super::put(out, part.heading())?;
            lines.iter().try_for_each(|line| super::put(out, line))?;
            said.push(part.next());
        }
        match said.is_empty() {
            true => super::put(out, NOTHING),
            false => super::put(err, &format!("celis: {}.", said.join("; "))),
        }
    }

    /// The live patterns in scope that fit `ask` by its `files`, its words or its meaning: a pattern with a
    /// `replaces` tag first, then those of a named file, of a glob, and the rest.
    pub(super) fn fits<'a>(
        &self,
        trails: &'a Trails,
        ask: &str,
        files: &[String],
        err: &mut dyn Write,
    ) -> Result<Vec<(&'a Note, Pattern)>, Failure> {
        let project = self.project();
        let paths: Vec<String> = files
            .iter()
            .flat_map(|f| super::matched::paths(&project, &self.folder, f))
            .collect();
        let live = trails.patterns().live(&project.anchor.project);
        let ids: HashSet<String> = live.iter().map(|(note, _)| short(&note.id)).collect();
        let meant: HashSet<String> = self
            .meant(ask, (Kind::Pattern, &ids), err)?
            .into_iter()
            .collect();
        let asked = words(ask);
        let mut found: Vec<(&Note, Pattern)> = live
            .into_iter()
            .filter(|(note, pattern)| {
                pattern.fit(&paths).is_some()
                    || meant.contains(&short(&note.id))
                    || !words(when(&pattern.content)).is_disjoint(&asked)
            })
            .collect();
        found.sort_by_key(|(_, pattern)| {
            let fit = pattern.fit(&paths);
            (pattern.replaces.is_none(), fit.is_none(), fit)
        });
        Ok(found)
    }

    /// The done asks among `steps` most like `ask`: those alike in meaning first, then those whose
    /// title and trail's title share the most words with it, newest first.
    fn past<'a>(
        &self,
        ask: &str,
        steps: &[Step<'a>],
        err: &mut dyn Write,
    ) -> Result<Vec<Step<'a>>, Failure> {
        let done: Vec<Step> = steps
            .iter()
            .copied()
            .filter(|s| s.is_ask() && s.state_word() == "done")
            .collect();
        let ids: HashSet<String> = done.iter().map(|s| short(s.id())).collect();
        let meant = self.meant(ask, (Kind::Step, &ids), err)?;
        let asked = words(ask);
        let mut shared: Vec<(usize, Step)> = done
            .iter()
            .map(|s| {
                let mut titles = words(s.title());
                titles.extend(words(s.trail().title()));
                (titles.intersection(&asked).count(), *s)
            })
            .filter(|(n, s)| *n >= SHARED && !meant.contains(&short(s.id())))
            .collect();
        shared.sort_by_key(|(n, s)| {
            (
                std::cmp::Reverse(*n),
                std::cmp::Reverse(s.note().created_at),
            )
        });
        let by_id: HashMap<String, Step> = done.iter().map(|s| (short(s.id()), *s)).collect();
        let alike = meant.iter().filter_map(|id| by_id.get(id).copied());
        Ok(alike.chain(shared.into_iter().map(|(_, s)| s)).collect())
    }

    /// The patterns `search` gave on this machine.
    pub(crate) fn recalls(&self) -> Recalls {
        self.home().map_or_else(|_| Recalls::default(), |home| Recalls::read(&home))
    }
}

/// A part of what `search` prints.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    Rules,
    Patterns,
    Past,
}

impl Part {
    fn heading(self) -> &'static str {
        match self {
            Part::Rules => "rules",
            Part::Patterns => "patterns",
            Part::Past => "past",
        }
    }

    /// What the agent does with the part.
    fn next(self) -> &'static str {
        match self {
            Part::Rules => "keep each rule: it binds",
            Part::Patterns => {
                "at each choice the ask leaves open, weigh the forces of each pattern that bears on it, \
                 choose like its good examples and unlike its bad ones, and name the pattern you \
                 followed, or why you departed from it; a rule wins over a pattern"
            }
            Part::Past => "take each past step that fits as yours, with step add --uses <its id>",
        }
    }
}

/// The lines of the done ask `ask`: its own, then each step right under it with its `changed` line, or
/// the ask itself as its one step when none is under it.
fn lines(ask: Step) -> Vec<String> {
    let mut lines = vec![format!(
        "{} {} · {} [{}]",
        short(ask.id()),
        flat(ask.title()),
        date(ask.note().created_at),
        ask.record().anchor.name
    )];
    let children = ask.children();
    let steps = match children.is_empty() {
        true => vec![ask],
        false => children,
    };
    for step in steps {
        let state = step.state_word();
        let title = flat(step.title());
        lines.push(format!("  step {} {state} {title}", short(step.id())));
        lines.extend(step.changed().map(|c| format!("    changed {}", flat(c))));
    }
    lines
}

/// What `search` prints when nothing fits.
const NOTHING: &str = "nothing fits this ask";

/// What `search` prints by meaning when no model was set up.
const NONE: &str = "celis search has no model: search with celis grep, or run celis setup.";

/// Words too common to tell patterns apart.
const COMMON: &[&str] = &[
    "about", "after", "all", "also", "always", "and", "any", "are", "before", "but", "can", "did",
    "does", "done", "each", "for", "from", "has", "have", "how", "into", "its", "just", "keep",
    "make", "more", "most", "must", "never", "new", "not", "one", "only", "other", "our", "out",
    "over", "per", "should", "some", "such", "than", "that", "the", "their", "them", "then",
    "there", "they", "this", "use", "via", "was", "were", "what", "when", "where", "which", "who",
    "why", "will", "with", "would", "you", "your",
];

/// The most smaller patterns `search` prints under a pattern.
const NEXT: usize = 5;

/// The most past asks `search` prints.
const PAST: usize = 2;

/// The least words a past ask's title shares with the ask to be like it.
const SHARED: usize = 2;

/// The part of a pattern's first line before its first `:`: the case it acts on.
fn when(content: &str) -> &str {
    let first = content.lines().next().unwrap_or("");
    first.split_once(':').map_or(first, |(when, _)| when)
}

/// The words of `text` that tell cases apart: 4 or more letters, not common, lower case, without a
/// plural `s`.
fn words(text: &str) -> HashSet<String> {
    let lower = text.to_lowercase();
    lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 4 && !COMMON.contains(w))
        .map(|w| w.strip_suffix('s').unwrap_or(w).to_string())
        .collect()
}

/// `text` on one line.
fn flat(text: &str) -> String {
    let lines = text.split(['\n', '\r']).filter(|l| !l.is_empty());
    lines.collect::<Vec<_>>().join(" ")
}
