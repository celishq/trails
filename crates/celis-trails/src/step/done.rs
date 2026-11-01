//! A step closed done.

use crate::By;
use celis_store::ProofFile;

/// A step closed done: its proof, what changed from its plan, who gave the proof, the files it keeps,
/// and the patterns that misled its work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Done {
    pub proof: String,
    pub changed: Option<String>,
    pub by: Option<By>,
    pub files: Vec<ProofFile>,
    pub misled: Vec<String>,
}

impl Done {
    /// A done with `proof` and what `changed`, given by the agent, with no file.
    pub fn new(proof: String, changed: Option<String>) -> Done {
        Done {
            proof,
            changed,
            by: None,
            files: Vec::new(),
            misled: Vec::new(),
        }
    }
}
