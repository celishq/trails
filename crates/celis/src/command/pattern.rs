//! Patterns: `celis pattern` writes one, and its parts read the index, the tidy list and the learn
//! brief.

pub(crate) mod candidate;
pub(crate) mod extract;
pub(crate) mod patterns;

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_patterns::pattern_record::when;
use celis_patterns::{Example, Examples, Pattern, PatternRecord, PatternStore, Scope};
use celis_store::records::id::short;
use celis_store::{PatternId, Place, Project, Source, Store};
use celis_trails::{TrailStore, Written};
use std::collections::HashSet;
use std::io::Write;

impl Command {
    /// Writes the pattern `args` states on `project`, in its scope, with its paths, labels and
    /// examples, signed by the account now, and prints its id and the live patterns like it.
    pub(crate) fn pattern(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        let content = args.required_text("pattern needs its text after --")?;
        let scope = match args.scope.as_deref() {
            Some(text) if text.starts_with("trail:") => return Err(Failure::usage(TRAIL)),
            Some(text) => Scope::parse(text)
                .ok_or_else(|| Failure::usage(format!("a scope is me or project, not {text}")))?,
            None => Scope::Project,
        };
        let paths: Vec<String> = args
            .paths
            .iter()
            .map(|p| Place::resolve(p, &project.folder, &self.folder))
            .collect();
        let trails = self.trails()?;
        let side = |values: &[String]| -> Result<Vec<Example>, Failure> {
            let found = values.iter().map(|v| self.example(&trails, project, v));
            found.collect()
        };
        let examples = Examples {
            good: side(&args.good)?,
            bad: side(&args.bad)?,
        };
        let pattern = PatternRecord {
            anchor: project.anchor.clone(),
            scope,
            replaces: None,
            content,
            paths,
            old_rule: false,
            within: args.within.iter().map(PatternId::new).collect(),
            examples,
        };
        let patterns = trails.patterns();
        let replaced = args.replaces.as_deref();
        let replaced = replaced.and_then(|id| patterns.view(&project.anchor.project).find(id).ok());
        let like = like(
            &patterns.view(&project.anchor.project).live(),
            &pattern.content,
            replaced,
        );
        let written = Written {
            records: vec![trails.decide(pattern, args.replaces.as_deref())?],
            ..Written::default()
        };
        let new = short(&written.records[0].id);
        self.print(written, args, out, err)?;
        for pattern in &like {
            let first = pattern.record.content.lines().next().unwrap_or("");
            let title = pattern.record.name().unwrap_or(first);
            crate::command::put(out, &format!("like    {} {title}", short(pattern.id())))?;
        }
        if let Some(pattern) = like.first() {
            let next = format!(
                "celis: the patterns above are like {new}. When {new} says the same as one, drop \
                 it with celis step drop {new} -- <reason>; when it changes one, drop it and write \
                 it with celis pattern --replaces {} -- <text>.",
                short(pattern.id())
            );
            crate::command::put(err, &next)?;
        }
        match args.scope {
            Some(_) => Ok(()),
            None => crate::command::put(err, SCOPES),
        }
    }

    /// The example `value` names: a URL, else a file of the working folder, stdin or a `data:` URI,
    /// kept as a blob, else a step of the store.
    fn example(&self, trails: &Store, project: &Project, value: &str) -> Result<Example, Failure> {
        if value.contains("://") {
            return Ok(Example::Url(value.to_string()));
        }
        let source = Source::of(value, &self.folder);
        if source.is_missing() {
            return Ok(Example::Step(trails.steps().find(value)?.step_id()));
        }
        let place = source.place(&project.folder, &self.folder);
        Ok(Example::File(trails.keep(place, &source)?))
    }
}

/// At most `LIKE` of `live`, other than `replaced`, whose "When" shares `SHARED` or more words with the
/// "When" of `content`, those that share the most first.
pub(crate) fn like<'a>(
    live: &[Pattern<'a>],
    content: &str,
    replaced: Option<Pattern>,
) -> Vec<Pattern<'a>> {
    let asked = words(when(content));
    let mut found: Vec<(usize, &Pattern<'a>)> = live
        .iter()
        .filter(|p| replaced.as_ref().is_none_or(|r| r.id() != p.id()))
        .map(|p| {
            (
                words(when(&p.record.content)).intersection(&asked).count(),
                p,
            )
        })
        .filter(|(n, _)| *n >= SHARED)
        .collect();
    found.sort_by_key(|(n, _)| std::cmp::Reverse(*n));
    found
        .into_iter()
        .take(LIKE)
        .map(|(_, p)| p.clone())
        .collect()
}

/// The most live patterns like the new one that `pattern` prints.
const LIKE: usize = 3;

/// The least words a live pattern's "When" shares with the new one's to be like it.
const SHARED: usize = 2;

/// What `pattern` with no scope says of the pattern's scope.
const SCOPES: &str = "celis: the pattern holds in this project. When it holds for that kind of work in any \
                      project, such as a pattern about a framework, a platform or a review's convention, \
                      replace it with --scope me; when it holds only for this trail, drop it and write \
                      it with celis step note <id>.";

/// What `pattern` with a `trail:<id>` scope says.
const TRAIL: &str =
    "a pattern holds in the project or for me; write what holds only for one trail with \
                     celis step note <id> -- <text>";

/// Words too common to tell patterns apart.
const COMMON: &[&str] = &[
    "about", "after", "all", "also", "always", "and", "any", "are", "before", "but", "can", "did",
    "does", "done", "each", "for", "from", "has", "have", "how", "into", "its", "just", "keep",
    "make", "more", "most", "must", "never", "new", "not", "one", "only", "other", "our", "out",
    "over", "per", "should", "some", "such", "than", "that", "the", "their", "them", "then",
    "there", "they", "this", "use", "via", "was", "were", "what", "when", "where", "which", "who",
    "why", "will", "with", "would", "you", "your",
];

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
