use serde::Deserialize;

/// The limits a relay holds a client to. A caller that needs either may treat silence as a no — unlike the
/// kinds, which are only advised on.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Limitation {
    /// Whether this relay serves nothing until a client has answered its challenge (NIP-42).
    #[serde(default)]
    pub auth_required: bool,
    /// The most notes it answers one subscription at once, or `None` when it has not said.
    #[serde(default)]
    pub max_limit: Option<u32>,
}
