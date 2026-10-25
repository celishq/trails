//! `celis recall`: the rules and patterns that fit an ask.

use super::rules::index::COMMON;
use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_nostr::Note;
use celis_trails::records::id::short;
use celis_trails::Rule;
use std::collections::HashSet;
use std::io::Write;

impl Command {
    /// Prints the live rules in scope that fit the ask of `args`, by its files, its words or its
    /// meaning, then the patterns that fit it with their examples. Writes no record.
    pub(crate) fn recall(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> Result<(), Failure> {
        let ask = args.required_text("recall needs the ask after --")?;
        if !self.held()? {
            return super::put(out, NOTHING);
        }
        let project = self.project();
        let paths: Vec<String> = args.words[1..]
            .iter()
            .flat_map(|f| super::rules::paths(&project, &self.folder, f))
            .collect();
        let meant: HashSet<String> = self.meant(&ask, err)?.into_iter().collect();
        let asked = words(&ask);
        let trails = self.trails()?;
        let live = trails.rules().live(&project.anchor.project).into_iter();
        let found: Vec<(&Note, Rule)> = live
            .filter(|(note, rule)| {
                rule.fit(&paths).is_some()
                    || meant.contains(&short(&note.id))
                    || !words(when(&rule.content)).is_disjoint(&asked)
            })
            .collect();
        if found.is_empty() {
            return super::put(out, NOTHING);
        }
        for (heading, patterns) in [("rules", false), ("patterns", true)] {
            let part: Vec<_> = found
                .iter()
                .filter(|(_, rule)| rule.is_pattern() == patterns)
                .collect();
            if part.is_empty() {
                continue;
            }
            super::put(out, heading)?;
            for (note, rule) in part {
                super::put(out, &crate::text::line::rule(note, rule, &trails))?;
            }
        }
        Ok(())
    }
}

/// What `recall` prints when no rule or pattern fits.
const NOTHING: &str = "nothing fits this ask";

/// The part of a rule's text before its first `:`: the case it acts on.
fn when(content: &str) -> &str {
    content.split_once(':').map_or(content, |(when, _)| when)
}

/// The words of `text` that tell cases apart: 4 or more letters, not common, lower case, without a
/// plural `s`.
fn words(text: &str) -> HashSet<String> {
    let lower = text.to_lowercase();
    lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 4 && !COMMON.contains(w))
        .map(|w| w.strip_suffix('s').unwrap_or(w).to_string())
        .collect()
}
