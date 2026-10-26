//! `celis pattern`: write a pattern.

use crate::command::read::search::{when, words};
use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_patterns::{Cell, Example, Examples, Pattern, PatternRecord, PatternStore, Scope};
use celis_store::records::id::short;
use celis_store::{PatternId, Place, Project, Store};
use celis_trails::{TrailStore, Written};
use std::io::Write;

impl Command {
    /// Writes the pattern `args` states on `project`, in its scope, with its trigger, labels, cells and
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
        if let Some(cell) = args.cells.iter().find(|c| Cell::new(c).is_none()) {
            return Err(Failure::usage(format!(
                "a cell needs a letter or a digit, not {cell:?}"
            )));
        }
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
            cells: Cell::all(args.cells.iter().map(String::as_str)),
            examples,
        };
        let patterns = trails.patterns();
        let replaced = args.replaces.as_deref();
        let replaced =
            replaced.and_then(|id| patterns.find(Some(&project.anchor.project), id).ok());
        let like = like(
            &patterns.live(&project.anchor.project),
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
            crate::command::put(out, &format!("like    {} {first}", short(pattern.id())))?;
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

    /// The example `value` names: a URL, else a file of the working folder, kept as a blob, else a
    /// step of the store.
    fn example(&self, trails: &Store, project: &Project, value: &str) -> Result<Example, Failure> {
        if value.contains("://") {
            return Ok(Example::Url(value.to_string()));
        }
        let path = self.folder.join(value);
        if path.is_file() {
            let place = Place::resolve(value, &project.folder, &self.folder);
            return Ok(Example::File(trails.keep(place, &path)?));
        }
        Ok(Example::Step(trails.steps().find(value)?.step_id()))
    }
}

/// At most `LIKE` of `live`, other than `replaced`, whose "When" shares `SHARED` or more words with the
/// "When" of `content`, those that share the most first.
fn like<'a>(live: &[Pattern<'a>], content: &str, replaced: Option<Pattern>) -> Vec<Pattern<'a>> {
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
