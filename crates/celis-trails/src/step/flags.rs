//! What a step or trail carries beside its state.

/// What a step or trail carries beside its state: `changed` when its done has a `changed` tag, and
/// `revised` when a later step replaces its done work. A trail reads its first ask's done, and is revised
/// when a step of its tree is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flags {
    pub changed: bool,
    pub revised: bool,
}
