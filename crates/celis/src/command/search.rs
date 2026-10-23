//! `celis search`: the rules and steps most alike in meaning to a text.

mod decoder;
mod encoder;
mod index;
mod model;
mod setup;
mod vectors;

use super::view::kind::Kind;
use super::view::row::Row;
use super::view::View;
use super::{Args, Command};
use crate::failure::Failure;
use index::Index;
use model::Model;
use serde_json::json;
use std::collections::HashSet;
use std::io::{IsTerminal, Write};

/// What `search` prints when no model was set up.
const NONE: &str = "celis search has no model: search with celis grep, or run celis setup.";

impl Command {
    /// Prints the live rules in scope and the trails and steps most alike in meaning to the text of
    /// `args`, by the chosen model; first it indexes the records the model has not read. Writes no
    /// record.
    pub(crate) fn search(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> Result<(), Failure> {
        let text = args.required_text("search needs a text after --")?;
        let kinds = match args.kind.as_deref() {
            None => vec![Kind::Rule, Kind::Step, Kind::Trail],
            Some("rule") => vec![Kind::Rule],
            Some("step") => vec![Kind::Step],
            Some("trail") => vec![Kind::Trail],
            Some(kind) => {
                return Err(Failure::usage(format!(
                    "--kind takes rule, step or trail, not {kind}"
                )))
            }
        };
        let limit = match args.limit.as_deref().map(str::parse::<usize>) {
            None => 20,
            Some(Ok(n)) if n > 0 => n,
            Some(_) => return Err(Failure::usage("--limit takes a positive number")),
        };
        let Some(model) = Model::chosen(&self.home()?) else {
            return super::put(out, NONE);
        };
        let rows = self.searched()?;
        let index = self.indexed(model, &rows, err)?;
        let alike = index.alike(&text)?;
        let mut found: Vec<(f32, &Row)> = rows
            .iter()
            .filter(|row| kinds.contains(&row.kind))
            .filter_map(|row| Some((alike(&row.text)?, row)))
            .collect();
        found.sort_by(|(a, _), (b, _)| b.total_cmp(a));
        for (score, row) in found.into_iter().take(limit) {
            let object = json!({"id": row.id, "kind": row.kind.text(), "score": score,
                "project": row.project, "content": row.text});
            let line = format!("{score:.2} {}", row.line());
            super::put(out, &if args.json { object.to_string() } else { line })?;
        }
        Ok(())
    }

    /// The rows `search` gives: each trail and step of every project, and each live rule in scope.
    fn searched(&self) -> Result<Vec<Row>, Failure> {
        if !self.held()? {
            return Ok(vec![]);
        }
        let trails = self.trails()?;
        let project = self.project();
        let live = trails.rules().live(&project.anchor.project).into_iter();
        let rules: HashSet<String> = live
            .map(|(note, _)| celis_trails::records::id::short(&note.id))
            .collect();
        let rows = View::of(&trails).into_rows().into_iter();
        Ok(rows
            .filter(|row| match row.kind {
                Kind::Trail | Kind::Step => true,
                Kind::Rule => rules.contains(&row.id),
                _ => false,
            })
            .collect())
    }

    /// The index of `model`, refreshed with the text of each of `rows`; on a terminal, the count read
    /// so far goes to `err`.
    fn indexed(
        &self,
        model: &'static Model,
        rows: &[Row],
        err: &mut dyn Write,
    ) -> Result<Index, Failure> {
        let mut index = Index::open(model, &self.home()?)?;
        let texts: Vec<&str> = rows.iter().map(|r| r.text.as_str()).collect();
        let progress = std::io::stderr().is_terminal().then_some(err);
        index.refresh(&texts, progress)?;
        Ok(index)
    }
}
