//! A step closed done.

use crate::By;
use celis_store::ProofFile;

/// A step closed done: its proof, what changed from its plan, who gave the proof, and the files it
/// keeps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Done {
    pub proof: String,
    pub changed: Option<String>,
    pub by: Option<By>,
    pub files: Vec<ProofFile>,
}

impl Done {
    /// A done with `proof` and what `changed`, given by the agent, with no file.
    pub fn new(proof: String, changed: Option<String>) -> Done {
        Done {
            proof,
            changed,
            by: None,
            files: Vec::new(),
        }
    }
}
