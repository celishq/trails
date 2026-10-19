//! CIP-07: the name of a trail, and the trail that holds it.

pub mod named_trail;

/// A name a trail goes by: a lowercase letter, then lowercase letters, digits and `-`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(String);

impl Name {
    /// The name `text` is; `None` when it is no name.
    pub fn parse(text: &str) -> Option<Name> {
        let mut chars = text.chars();
        let first = chars.next().is_some_and(|c| c.is_ascii_lowercase());
        let rest = chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        (first && rest).then(|| Name(text.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
