//! What `celis search` finds by meaning, through the model that `setup` chose.

use super::NONE;
use crate::command::view::kind::Kind;
use crate::command::view::row::Row;
use crate::command::view::View;
use crate::command::Command;
use crate::failure::Failure;
use celis_search::{Index, Model};
use serde_json::json;
use std::collections::HashSet;
use std::io::{IsTerminal, Write};

/// The most records `search` takes by meaning for a part.
const MEANT: usize = 5;

/// The least cosine at which `search` takes a record by meaning for a part.
const CLOSE: f32 = 0.3;

impl Command {
    /// Prints at most `limit` records of `kind` most alike in meaning to `text`, highest first, as
    /// lines or as `json`; first it indexes the records the model has not read.
    pub(super) fn listed(
        &self,
        text: &str,
        kind: Kind,
        (limit, json): (usize, bool),
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> Result<(), Failure> {
        let Some(model) = Model::chosen(&self.home()?) else {
            return crate::command::put(out, NONE);
        };
        let rows = self.searched()?;
        let index = self.indexed(model, &rows, err)?;
        let alike = index.alike(text)?;
        let mut found: Vec<(f32, &Row)> = rows
            .iter()
            .filter(|row| row.kind == kind)
            .filter_map(|row| Some((alike(&row.text)?, row)))
            .collect();
        found.sort_by(|(a, _), (b, _)| b.total_cmp(a));
        for (score, row) in found.into_iter().take(limit) {
            let object = json!({"id": row.id, "kind": row.kind.text(), "score": score,
                "project": row.project, "content": row.text});
            let line = format!("{score:.2} {}", row.line());
            crate::command::put(out, &if json { object.to_string() } else { line })?;
        }
        Ok(())
    }

    /// The short ids of the records of `kind` among `ids` most alike in meaning to `text`, at most
    /// `MEANT`, each at least `CLOSE`, most alike first; none with no model.
    pub(super) fn meant(
        &self,
        text: &str,
        (kind, ids): (Kind, &HashSet<String>),
        err: &mut dyn Write,
    ) -> Result<Vec<String>, Failure> {
        let Some(model) = Model::chosen(&self.home()?) else {
            return Ok(vec![]);
        };
        let rows: Vec<Row> = self
            .searched()?
            .into_iter()
            .filter(|r| r.kind == kind && ids.contains(&r.id))
            .collect();
        let index = self.indexed(model, &rows, err)?;
        let alike = index.alike(text)?;
        let mut found: Vec<(f32, &Row)> = rows
            .iter()
            .filter_map(|row| Some((alike(&row.text)?, row)))
            .filter(|(score, _)| *score >= CLOSE)
            .collect();
        found.sort_by(|(a, _), (b, _)| b.total_cmp(a));
        Ok(found
            .into_iter()
            .take(MEANT)
            .map(|(_, r)| r.id.clone())
            .collect())
    }

    /// The rows `search` reads: each trail and step of every project, and each live pattern in scope.
    pub(super) fn searched(&self) -> Result<Vec<Row>, Failure> {
        if !self.held()? {
            return Ok(vec![]);
        }
        let trails = self.trails()?;
        let project = self.project();
        let live = trails.patterns().live(&project.anchor.project).into_iter();
        let patterns: HashSet<String> = live
            .map(|(note, _)| celis_trails::records::id::short(&note.id))
            .collect();
        let rows = View::of(&trails).into_rows().into_iter();
        Ok(rows
            .filter(|row| match row.kind {
                Kind::Trail | Kind::Step => true,
                Kind::Pattern => patterns.contains(&row.id),
                _ => false,
            })
            .collect())
    }

    /// The index of `model`, refreshed with the text of each of `rows`; on a terminal, the count read
    /// so far goes to `err`.
    pub(super) fn indexed(
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
