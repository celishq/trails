//! How the work of one closed step went.

use serde::Serialize;

/// How the work of one closed step went: the seconds it took.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Grade {
    pub took: u64,
}
