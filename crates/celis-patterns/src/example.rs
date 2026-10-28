//! One example of a pattern.

use celis_store::{ProofFile, StepId};

/// One example of a pattern: a done step, a file kept as a blob, or a URL (CIP-05, "Patterns").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Example {
    Step(StepId),
    File(ProofFile),
    Url(String),
}

impl Example {
    /// Its tag after the name `side`, with `pubkey` as the account that keeps a file.
    pub fn tag(&self, side: &str, pubkey: &str) -> Vec<String> {
        let mut tag = vec![side.to_string()];
        match self {
            Example::Step(id) => tag.push(id.to_string()),
            Example::File(file) => {
                tag.push(file.url(pubkey));
                if !file.place.is_empty() {
                    tag.push(file.place.clone());
                }
            }
            Example::Url(url) => tag.push(url.clone()),
        }
        tag
    }

    /// The example the values of a `good` or `bad` tag name.
    pub fn read(values: &[String]) -> Option<Example> {
        let value = values.first()?;
        if let Some(file) = ProofFile::blossom(values) {
            return Some(Example::File(file));
        }
        match value.contains("://") {
            true => Some(Example::Url(value.clone())),
            false => Some(Example::Step(StepId::new(value))),
        }
    }
}
