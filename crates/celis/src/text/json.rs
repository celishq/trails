//! The `--json` object of each record.

use celis_nostr::Note;
use celis_trails::{Rule, StepRecord};
use serde_json::{json, Value};

/// The object of the step `record` that `note` holds.
pub(crate) fn step(note: &Note, record: &StepRecord) -> Value {
    json!({
        "id": note.id,
        "created_at": note.created_at,
        "project": record.anchor.project,
        "links": links(record),
        "places": record.places,
        "check": record.check,
        "content": record.content,
    })
}

/// The links of the step `record`: its `under`, `replaces` and each `uses`, in that order.
pub(crate) fn links(record: &StepRecord) -> Value {
    let one = [("under", &record.under), ("replaces", &record.replaces)];
    let one = one
        .into_iter()
        .filter_map(|(kind, to)| Some((kind, &**to.as_ref()?)));
    let uses = record.uses.iter().map(|to| ("uses", to.as_str()));
    let links = one
        .chain(uses)
        .map(|(kind, to)| json!({"to": to, "kind": kind}));
    Value::Array(links.collect())
}

/// The object of the rule that `note` holds.
pub(crate) fn rule(note: &Note, rule: &Rule) -> Value {
    json!({
        "id": note.id,
        "created_at": note.created_at,
        "project": rule.anchor.project,
        "scope": rule.scope.text(),
        "replaces": rule.replaces,
        "content": rule.content,
    })
}
