//! How a done step went against its plan.

/// How a done step went against its plan: its expect line and its own changed, the steps under it planned
/// before the first started, added after, dropped with their reasons and changed, the person's prompts on
/// its tree, and the open steps beside it under `under`, each as an id and a title.
pub struct Review {
    pub step: String,
    pub under: Option<String>,
    pub ask: bool,
    pub expect: Option<String>,
    pub as_planned: bool,
    pub planned: usize,
    pub added: Vec<(String, String)>,
    pub dropped: Vec<(String, String, String)>,
    pub changed: Vec<(String, String)>,
    pub asked: Vec<String>,
    pub open: Vec<(String, String)>,
}
