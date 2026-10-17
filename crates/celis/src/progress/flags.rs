//! What a step or trail carries beside its state.

use celis_trails::Step;

/// What a step or trail carries beside its state: `changed` when its done has a `changed` tag, and
/// `revised` when a later step replaces its done work. A trail reads its first ask's done, and is revised
/// when a step of its tree is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flags {
    pub changed: bool,
    pub revised: bool,
}

impl Flags {
    pub fn of(step: Step) -> Flags {
        let revised = match step.is_trail() {
            true => !step.reworked().is_empty(),
            false => !step.revised().is_empty(),
        };
        Flags {
            changed: step.changed().is_some(),
            revised,
        }
    }
}
