//! What an id of a command names.

use crate::Step;
use celis_nostr::Note;
use celis_store::Refusal;

/// What an id names: a pattern or a step.
pub(crate) enum Target<'a> {
    Pattern(&'a Note),
    Step(Step<'a>),
}

impl<'a> Target<'a> {
    /// What `id` names, of the `pattern` and the `step` it starts; refuses both as a usage error, and
    /// neither with `no-step`.
    pub(crate) fn of(
        id: &str,
        pattern: Option<&'a Note>,
        step: Option<Step<'a>>,
    ) -> Result<Target<'a>, Refusal> {
        match (pattern, step) {
            (Some(_), Some(_)) => Err(Refusal::Usage(format!(
                "{id} starts the id of a pattern and of a step; give more of it"
            ))),
            (Some(pattern), None) => Ok(Target::Pattern(pattern)),
            (None, Some(step)) => Ok(Target::Step(step)),
            (None, None) => Err(Refusal::NoStep(id.to_string())),
        }
    }
}
