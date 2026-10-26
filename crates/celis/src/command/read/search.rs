//! `celis search`: the patterns and the past asks that fit an ask.

#[cfg(feature = "search")]
mod meaning;
#[cfg(not(feature = "search"))]
#[path = "search/off.rs"]
mod meaning;
pub(crate) mod recalls;
#[cfg(feature = "search")]
mod setup;

use crate::command::read::view::kind::Kind;
use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::text::clock::date;
use celis_patterns::{Pattern, PatternStore};
use celis_store::records::id::short;
use celis_store::Store;
use celis_trails::{Step, TrailStore};
use recalls::Recalls;
use std::collections::{HashMap, HashSet};
use std::io::Write;

impl Command {
    /// Prints the first live patterns in scope that fit the ask of `args` and a line that names the rest,
    /// each with its examples, then the done asks like it, and marks each printed pattern as recalled
    /// now; with `--kind`, only that part. Writes no record.
    pub(crate) fn search(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> Result<(), Failure> {
        let ask = args.required_text("search needs the ask after --")?;
        let limit = Args::number("--limit", &args.limit)?;
        let parts = match args.kind.as_deref() {
            None => vec![Part::Patterns, Part::Past],
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
                    "--kind takes pattern, trail or step, not {kind}"
                )))
            }
        };
        if args.json {
            return Err(Failure::usage("--json takes --kind trail or --kind step"));
        }
        if !self.held()? {
            return crate::command::put(out, NOTHING);
        }
        let trails = self.trails()?;
        let files = &args.words[1..];
        let found = self.fits(&trails, &ask, files, err)?;
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
                    .take(limit.unwrap_or(PAST).min(PAST))
                    .flat_map(|a| lines(*a))
                    .collect(),
                Part::Patterns => {
                    let all: Vec<&Pattern> = found.iter().collect();
                    let shown = limit.unwrap_or(BUDGET).min(all.len());
                    let part = &all[..shown];
                    if let Ok(home) = self.home() {
                        Recalls::mark(&home, part.iter().map(|pattern| pattern.note));
                    }
                    let project = self.project().anchor.project;
                    let line = |pattern: &&Pattern| {
                        let mut lines = vec![crate::text::line::pattern(pattern, &trails)];
                        let smaller = trails.patterns().smaller(&project, pattern.id());
                        lines.extend(smaller.iter().take(NEXT).map(|smaller| {
                            let first = smaller.record.content.lines().next().unwrap_or("");
                            format!("  next    {} {first}", short(smaller.id()))
                        }));
                        lines.join("\n")
                    };
                    let mut lines: Vec<String> = part.iter().map(line).collect();
                    if shown < all.len() {
                        let files: String = files.iter().map(|f| format!("{f} ")).collect();
                        lines.push(format!(
                            "{} more: celis search {files}--kind pattern --limit {} -- {ask:?}",
                            all.len() - shown,
                            all.len()
                        ));
                    }
                    lines
                }
            };
            if lines.is_empty() {
                continue;
            }
            crate::command::put(out, part.heading())?;
            lines
                .iter()
                .try_for_each(|line| crate::command::put(out, line))?;
            said.push(part.next());
        }
        match said.is_empty() {
            true => crate::command::put(out, NOTHING),
            false => crate::command::put(err, &format!("celis: {}.", said.join("; "))),
        }
    }

    /// The live patterns in scope that fit `ask` by its `files`, their cells, its words or its meaning, in
    /// the order `Patterns::fitting` gives.
    pub(crate) fn fits<'a>(
        &self,
        trails: &'a Store,
        ask: &str,
        files: &[String],
        err: &mut dyn Write,
    ) -> Result<Vec<Pattern<'a>>, Failure> {
        let project = self.project();
        let paths: Vec<String> = files
            .iter()
            .flat_map(|f| crate::command::record::files::paths(&project, &self.folder, f))
            .collect();
        let patterns = trails.patterns();
        let live = patterns.live(&project.anchor.project);
        let ids: HashSet<String> = live.iter().map(|p| short(p.id())).collect();
        let meant: HashSet<String> = self
            .meant(ask, (Kind::Pattern, &ids), err)?
            .into_iter()
            .collect();
        let whens: Vec<HashSet<String>> = live
            .iter()
            .map(|p| words(when(&p.record.content)))
            .collect();
        let named = words(ask);
        let asked: HashSet<String> = named
            .iter()
            .filter(|w| {
                let n = whens.iter().filter(|when| when.contains(*w)).count();
                n <= WIDE || n * 100 <= whens.len() * SHARE
            })
            .cloned()
            .collect();
        let filed: HashSet<String> = live
            .iter()
            .filter(|p| p.record.fit(&paths).is_some())
            .flat_map(|p| p.record.cells.iter().map(|c| c.name().to_string()))
            .collect();
        let cell = |p: &Pattern| {
            let cells = p.record.cells.iter();
            cells
                .map(|c| c.name())
                .any(|c| filed.contains(c) || !words(c).is_disjoint(&named))
        };
        Ok(
            patterns.fitting(&project.anchor.project, &paths, cell, |p| {
                meant.contains(&short(p.id()))
                    || !words(when(&p.record.content)).is_disjoint(&asked)
            }),
        )
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
        self.home()
            .map_or_else(|_| Recalls::default(), |home| Recalls::read(&home))
    }
}

/// A part of what `search` prints.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    Patterns,
    Past,
}

impl Part {
    fn heading(self) -> &'static str {
        match self {
            Part::Patterns => "patterns",
            Part::Past => "past",
        }
    }

    /// What the agent does with the part.
    fn next(self) -> &'static str {
        match self {
            Part::Patterns => {
                "at each choice the ask leaves open, weigh the forces of each pattern that bears on it, \
                 choose like its good examples and unlike its bad ones, and name the pattern you \
                 followed, or why you departed from it; a rule wins over a pattern"
            }
            Part::Past => "take each past step that fits as yours, with step add --uses <its id>",
        }
    }
}

/// The lines of the done ask `ask`: its own, then each `changed` line of the steps right under it, or of
/// the ask itself when none is under it.
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
        let id = short(step.id());
        lines.extend(
            step.changed()
                .map(|c| format!("  changed {id} {}", flat(c))),
        );
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

/// The most patterns `search` prints when no `--limit` is given.
const BUDGET: usize = 8;

/// The most patterns a word of the ask may be in and still fit, whatever their share.
const WIDE: usize = 5;

/// The most percent of the patterns a word of the ask may be in and still fit.
const SHARE: usize = 5;

/// The most past asks `search` prints.
const PAST: usize = 2;

/// The least words a past ask's title shares with the ask to be like it.
const SHARED: usize = 2;

/// The part of a pattern's first line before its first `:`: the case it acts on.
pub(crate) fn when(content: &str) -> &str {
    let first = content.lines().next().unwrap_or("");
    first.split_once(':').map_or(first, |(when, _)| when)
}

/// The words of `text` that tell cases apart: 4 or more letters, not common, lower case, without a
/// plural `s`.
pub(crate) fn words(text: &str) -> HashSet<String> {
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
