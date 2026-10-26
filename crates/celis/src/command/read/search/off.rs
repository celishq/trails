//! What `celis search` finds by meaning, and `celis setup`, in a build with no search feature.

use super::NONE;
use crate::command::read::view::kind::Kind;
use crate::command::{put, Args, Command};
use crate::failure::Failure;
use std::collections::HashSet;
use std::io::Write;

impl Command {
    /// Prints that search has no model.
    pub(super) fn listed(
        &self,
        _text: &str,
        _kind: Kind,
        _shown: (usize, bool),
        out: &mut dyn Write,
        _err: &mut dyn Write,
    ) -> Result<(), Failure> {
        put(out, NONE)
    }

    /// None: this build cannot run a model.
    pub(super) fn meant(
        &self,
        _text: &str,
        _among: (Kind, &HashSet<String>),
        _err: &mut dyn Write,
    ) -> Result<Vec<String>, Failure> {
        Ok(vec![])
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
