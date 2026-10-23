//! `celis search`: the rules and steps most alike in meaning to a text.

mod decoder;
mod encoder;
mod index;
pub(super) mod model;

use super::grep::kind::Kind;
use super::grep::row::Row;
use super::grep::view::View;
use super::{Args, Command};
use crate::failure::Failure;
use encoder::Encoder;
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
        let searched = self.searched()?;
        let (encoder, index) = self.indexed(model, &searched, err)?;
        let query = encoder.query(&text)?;
        let mut found: Vec<(f32, &Row)> = rows(&searched)
            .filter(|row| kinds.contains(&row.kind))
            .filter_map(|row| Some((dot(&query, index.get(&row.text)?), row)))
            .collect();
        found.sort_by(|(a, _), (b, _)| b.total_cmp(a));
        for (score, row) in found.into_iter().take(limit) {
            let object = json!({"id": row.id(), "kind": row.kind.text(), "score": score,
                "project": row.project, "content": row.text});
            let line = format!("{score:.2} {}", row.line());
            super::put(out, &if args.json { object.to_string() } else { line })?;
        }
        Ok(())
    }

    /// The view of the records `search` gives: every trail and step, and the live rules in scope.
    fn searched(&self) -> Result<Searched, Failure> {
        if !self.held()? {
            return Ok(Searched::default());
        }
        let trails = self.trails()?;
        let project = self.project();
        let live = trails.rules().live(&project.anchor.project).into_iter();
        let rules = live
            .map(|(note, _)| celis_trails::records::id::short(&note.id))
            .collect();
        Ok(Searched {
            view: View::of(&trails),
            rules,
        })
    }

    /// The model `model` loaded, and its index with a vector for each record of `searched`, saved;
    /// on a terminal, the count of records indexed so far goes to `err`.
    fn indexed(
        &self,
        model: &'static Model,
        searched: &Searched,
        err: &mut dyn Write,
    ) -> Result<(Encoder, Index), Failure> {
        let home = self.home()?;
        let encoder = Encoder::load(model, &model.folder(&home))?;
        let file = home.join("index").join(format!("{}.bin", model.id));
        let mut index = Index::open(&file, model.id, model.dimensions);
        let texts: Vec<&str> = rows(searched).map(|r| r.text.as_str()).collect();
        let missing: Vec<&str> = texts
            .iter()
            .copied()
            .filter(|t| index.get(t).is_none())
            .collect();
        let shown = std::io::stderr().is_terminal();
        for (n, text) in missing.iter().enumerate() {
            index.insert(text, encoder.document(text)?);
            if shown {
                let _ = write!(err, "\rcelis: indexing {}/{}", n + 1, missing.len());
            }
        }
        if shown && !missing.is_empty() {
            let _ = writeln!(err);
        }
        index.keep(texts);
        index.save(&file)?;
        Ok((encoder, index))
    }
}

/// The view, and the ids of the live rules in scope.
#[derive(Default)]
struct Searched {
    view: View,
    rules: HashSet<String>,
}

/// The rows of `searched` that `search` gives: each trail and step, and each live rule in scope.
fn rows(searched: &Searched) -> impl Iterator<Item = &Row> {
    searched.view.rows().iter().filter(|row| match row.kind {
        Kind::Trail | Kind::Step => true,
        Kind::Rule => searched.rules.contains(row.id()),
        _ => false,
    })
}

/// The cosine of two unit vectors.
fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
