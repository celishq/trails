//! `celis names`: the named trails, most used first.

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
    pub(crate) fn names(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
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
            .map(|named| (named, reuse.tree_followers(named.trail)))
            .collect();
        named.sort_by_key(|(named, uses)| {
            let trail = named.trail;
            (Reverse(*uses), Reverse(trail.note().created_at), trail.id())
        });
        for (named, uses) in named.into_iter().take(limit) {
            let (name, trail) = (named.name.as_str(), named.trail);
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
