//! A step as an agent asks to write it.

use celis_store::Anchor;

/// A step as an agent asks to write it: its words, its anchor, places and check, the ids it names as
/// given, and whether it is a new ask beside its session's current trail.
pub struct Draft {
    pub content: String,
    pub anchor: Anchor,
    pub places: Vec<String>,
    pub check: Option<String>,
    pub under: Option<String>,
    pub replaces: Option<String>,
    pub uses: Vec<String>,
    pub misled: Vec<String>,
    pub session: Option<String>,
    pub new: bool,
    pub agent: Option<String>,
}
