//! What an id of a command names.

use crate::{Refusal, Step};
use celis_nostr::Note;

/// What an id names: a rule or a step.
pub(crate) enum Target<'a> {
    Rule(&'a Note),
    Step(Step<'a>),
}

impl<'a> Target<'a> {
    /// What `id` names, of the `rule` and the `step` it starts; refuses both as a usage error, and
    /// neither with `no-step`.
    pub(crate) fn of(
        id: &str,
        rule: Option<&'a Note>,
        step: Option<Step<'a>>,
    ) -> Result<Target<'a>, Refusal> {
        match (rule, step) {
            (Some(_), Some(_)) => Err(Refusal::Usage(format!(
                "{id} starts the id of a rule and of a step; give more of it"
            ))),
            (Some(rule), None) => Ok(Target::Rule(rule)),
            (None, Some(step)) => Ok(Target::Step(step)),
            (None, None) => Err(Refusal::NoStep(id.to_string())),
        }
    }
}
