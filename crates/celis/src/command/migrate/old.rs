//! A live pattern of a retired form, and its rewrite in the current form.

use celis_nostr::Note;
use celis_patterns::{Examples, PatternRecord, Scope};
use celis_store::{Anchor, Drops, PatternId, Place, Refusal, Store};
use celis_trails::Mark;
use std::collections::HashSet;

/// A live pattern that older versions wrote in a form no reader reads: labelled `decision` in
/// `celis.decision`, or with an `at:<place>` or a `trail:<id>` scope (`spec/02-cli.md`, "Migrate").
pub(crate) struct Old<'a> {
    pub note: &'a Note,
    pub form: Form,
}

/// The retired form of a pattern.
pub(crate) enum Form {
    Decision,
    At(String),
    Trail(String),
}

impl<'a> Old<'a> {
    /// Each live pattern of a retired form in `records`, oldest first.
    pub fn every(records: &'a [Note]) -> Vec<Old<'a>> {
        let replaced: HashSet<&str> = records.iter().filter_map(|n| n.tag("replaces")).collect();
        let drops = Drops::of(records);
        let live = |n: &Note| !replaced.contains(n.id.as_str()) && drops.get(&n.id).is_none();
        let mut old: Vec<Old> = records
            .iter()
            .filter(|n| live(n))
            .filter_map(|note| {
                Some(Old {
                    note,
                    form: Form::of(note)?,
                })
            })
            .collect();
        old.sort_by(|a, b| (a.note.created_at, &a.note.id).cmp(&(b.note.created_at, &b.note.id)));
        old
    }

    /// Writes it again in the current form, signed now, and returns each record written.
    pub fn write(&self, store: &Store) -> Result<Vec<Note>, Refusal> {
        let note = self.note;
        if let Form::Trail(trail) = &self.form {
            let text = Mark::Note {
                text: note.content.clone(),
            };
            let reason = Mark::Drop {
                reason: "migrated to a note on its trail".to_string(),
            };
            return Ok(vec![
                store.sign(|account, now| text.record(account, trail, now))?,
                store.sign(|account, now| reason.record(account, &note.id, now))?,
            ]);
        }
        let mut paths: Vec<String> = note.values("path").map(str::to_string).collect();
        let scope = match &self.form {
            Form::At(place) => {
                let path = Place::new(place).path().trim_end_matches('/');
                if !matches!(path, "." | "") {
                    paths.extend([path.to_string(), format!("{path}/**")]);
                }
                Scope::Project
            }
            _ => note
                .tag("scope")
                .and_then(Scope::parse)
                .unwrap_or(Scope::Project),
        };
        let pattern = PatternRecord {
            anchor: Anchor::read(note).ok_or_else(|| Refusal::NoPattern(note.id.clone()))?,
            scope,
            replaces: Some(PatternId::new(&note.id)),
            content: note.content.clone(),
            paths,
            within: note.values("within").map(PatternId::new).collect(),
            examples: Examples::read(note),
        };
        Ok(vec![
            store.sign(|account, now| pattern.record(account, now))?
        ])
    }
}

impl Form {
    /// The retired form of the pattern `note` holds; `None` when it is no pattern, or a pattern of the
    /// current form.
    fn of(note: &Note) -> Option<Form> {
        let labelled = |label: &str, namespace: &str| note.labels(namespace).any(|l| l == label);
        let decision = labelled("decision", "celis.decision");
        if note.kind != 1111 || note.tag("K") != Some("project") {
            return None;
        }
        if !decision && !labelled("pattern", "celis.pattern") {
            return None;
        }
        let scope = note.tag("scope").unwrap_or("");
        if let Some(trail) = scope.strip_prefix("trail:").filter(|t| !t.is_empty()) {
            return Some(Form::Trail(trail.to_string()));
        }
        if let Some(place) = scope.strip_prefix("at:").filter(|p| !p.is_empty()) {
            return Some(Form::At(place.to_string()));
        }
        let known = note.tag("scope").is_none_or(|s| Scope::parse(s).is_some());
        (decision && known).then_some(Form::Decision)
    }

    /// Its name: `decision`, `at` or `trail`.
    pub fn name(&self) -> &'static str {
        match self {
            Form::Decision => "decision",
            Form::At(_) => "at",
            Form::Trail(_) => "trail",
        }
    }
}
