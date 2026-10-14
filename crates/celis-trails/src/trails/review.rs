//! How a done step went against its plan.

use crate::StepId;

/// How a done step went against its plan: its expect line and its own changed, the steps under it planned
/// before the first started, added after, dropped with their reasons and changed, the person's prompts on
/// its tree, and the open steps beside it under `under`, each as an id and a title.
pub struct Review {
    pub step: StepId,
    pub under: Option<StepId>,
    pub ask: bool,
    pub expect: Option<String>,
    pub as_planned: bool,
    pub planned: usize,
    pub added: Vec<(StepId, String)>,
    pub dropped: Vec<(StepId, String, String)>,
    pub changed: Vec<(StepId, String)>,
    pub asked: Vec<String>,
    pub open: Vec<(StepId, String)>,
}
