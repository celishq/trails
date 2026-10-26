//! `celis pattern`: write a pattern.

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_patterns::{Cell, Example, Examples, PatternRecord, PatternStore, Scope};
use celis_store::{PatternId, Place, Project, Store};
use celis_trails::{TrailStore, Written};
use std::io::Write;

impl Command {
    /// Writes the pattern `args` states on `project`, in its scope, with its trigger, labels, cells and
    /// examples, signed by the account now, and prints its id.
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
            binds: args.binds,
            within: args.within.iter().map(PatternId::new).collect(),
            cells: Cell::all(args.cells.iter().map(String::as_str)),
            examples,
        };
        let written = Written {
            records: vec![trails.decide(pattern, args.replaces.as_deref())?],
            ..Written::default()
        };
        self.print(written, args, out, err)?;
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

/// What `pattern` with no scope says of the pattern's scope.
const SCOPES: &str = "celis: the rule holds in this project. When it holds for that kind of work in any \
                      project, such as a rule about a framework, a platform or a review's convention, \
                      replace it with --scope me; when it holds only for this trail, drop it and write \
                      it with celis step note <id>.";

/// What `pattern` with a `trail:<id>` scope says.
const TRAIL: &str =
    "a pattern holds in the project or for me; write what holds only for one trail with \
                     celis step note <id> -- <text>";
