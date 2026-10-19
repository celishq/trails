//! `celis skills`: the named trails, most used first.

use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::progress::Reuse;
use celis_trails::records::id::short;
use serde_json::json;
use std::cmp::Reverse;
use std::io::Write;

impl Command {
    /// Prints each named trail of every project, most used first, then newest, up to `--limit`.
    /// Writes no record.
    pub(crate) fn skills(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let limit = match &args.limit {
            Some(text) => text
                .parse()
                .map_err(|_| Failure::usage(format!("--limit takes a number, not {text}")))?,
            None => usize::MAX,
        };
        if !self.held()? {
            return Ok(());
        }
        let trails = self.trails()?;
        let steps = trails.steps();
        let reuse = Reuse::new(&steps);
        let mut named: Vec<_> = steps
            .names()
            .into_iter()
            .map(|(name, trail)| {
                let tree = trail.tree().into_iter();
                (
                    name,
                    trail,
                    tree.map(|s| reuse.users(s).len()).sum::<usize>(),
                )
            })
            .collect();
        named.sort_by_key(|(_, trail, uses)| {
            (Reverse(*uses), Reverse(trail.note().created_at), trail.id())
        });
        for (name, trail, uses) in named.into_iter().take(limit) {
            let name = name.as_str();
            match args.json {
                true => {
                    let object = json!({"name": name, "id": trail.id(), "title": trail.title(), "uses": uses});
                    super::put(out, &object.to_string())?
                }
                false => super::put(
                    out,
                    &format!("{name} {} {}", short(trail.id()), trail.title()),
                )?,
            }
        }
        Ok(())
    }
}
