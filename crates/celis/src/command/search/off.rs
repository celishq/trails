//! `celis search` and `celis setup` in a build with no search feature.

use crate::command::view::row::Row;
use crate::command::{put, Args, Command};
use crate::failure::Failure;
use std::io::Write;

impl Command {
    /// Prints that search has no model. Writes no record.
    pub(crate) fn search(
        &self,
        args: &Args,
        out: &mut dyn Write,
        _err: &mut dyn Write,
    ) -> Result<(), Failure> {
        args.required_text("search needs a text after --")?;
        put(
            out,
            "celis search has no model: search with celis grep, or run celis setup.",
        )
    }

    /// None: this build has no model to find alike trails with.
    pub(super) fn alike(
        &self,
        _text: &str,
        _n: usize,
        _err: &mut dyn Write,
    ) -> Result<Option<Vec<Row>>, Failure> {
        Ok(None)
    }

    /// Fails: this build cannot run a model.
    pub(crate) fn setup(
        &self,
        _args: &Args,
        _out: &mut dyn Write,
        _err: &mut dyn Write,
    ) -> Result<(), Failure> {
        Err(anyhow::anyhow!("this celis was built with no search feature").into())
    }
}
