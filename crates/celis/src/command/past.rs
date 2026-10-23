//! `celis past`: what the past agent reads at a new ask.

use super::{put, Args, Command};
use crate::failure::Failure;
use std::io::Write;

/// The most rules one page holds.
const PAGE: usize = 50;

/// The trails alike to the ask that the first page shows.
const TRAILS: usize = 3;

impl Command {
    /// Prints page `--page`, else the first, of the live rules in scope; the first page also shows the
    /// lines of the trails most alike in meaning to the ask, and names the other pages. Writes no
    /// record.
    pub(crate) fn past(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> Result<(), Failure> {
        let page = Args::number("--page", &args.page)?.unwrap_or(1);
        let ask = match page {
            1 => Some(args.required_text("past needs the ask after --")?),
            _ => None,
        };
        let lines: Vec<String> = match self.held()? {
            true => {
                let trails = self.trails()?;
                let live = trails.rules().live(&self.project().anchor.project);
                live.into_iter()
                    .map(|(note, rule)| crate::text::line::rule(note, &rule))
                    .collect()
            }
            false => vec![],
        };
        let pages = lines.len().div_ceil(PAGE).max(1);
        if page > pages {
            return Err(Failure::usage(format!(
                "--page takes a number from 1 to {pages}"
            )));
        }
        put(out, &format!("rules, page {page} of {pages}"))?;
        for line in lines.iter().skip((page - 1) * PAGE).take(PAGE) {
            put(out, &format!("  {line}"))?;
        }
        let Some(ask) = ask else {
            return Ok(());
        };
        put(out, "past")?;
        match self.alike(&ask, TRAILS, err)? {
            None => put(
                out,
                "  celis search has no model: find past asks with celis grep.",
            )?,
            Some(rows) if rows.is_empty() => put(out, "  nothing")?,
            Some(rows) => {
                for row in rows {
                    put(out, &format!("  {}", row.line()))?;
                }
            }
        }
        if pages > 1 {
            let next = format!(
                "celis: read pages 2 to {pages} with celis past --page <n>, all in one message."
            );
            put(out, &next)?;
        }
        Ok(())
    }
}
