//! How the work of one closed step went.

use crate::StepId;
use serde::Serialize;

/// How the work of one closed step went: the seconds it took, and the steps that replaced it after.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Grade {
    pub took: u64,
    pub revised: Vec<StepId>,
}
