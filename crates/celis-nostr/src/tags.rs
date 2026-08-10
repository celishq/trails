//! **What a note states beside its content** — NIP-01's tags, in the order they were stated.
//!
//! A tag is a list of strings whose first is its name, so the shape here is deliberately the wire's own: a
//! name with one value covers nearly everything, a name on its own is NIP-70's `-`, and anything longer is
//! built as the list it is. Nothing is deduplicated or reordered — a note's id covers its tags exactly as
//! they were written.

/// The tags a note carries, built in the order a reader will see them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tags(Vec<Vec<String>>);

impl Tags {
    pub fn new() -> Self {
        Tags(Vec::new())
    }

    /// The same tags, and `name` stating `value`.
    pub fn tag(mut self, name: &str, value: impl Into<String>) -> Self {
        self.0.push(vec![name.to_string(), value.into()]);
        self
    }

    /// The same tags, and `name` stating nothing beside it — NIP-70's `-`, which says a note belongs to the
    /// relay it was sent to and nowhere else.
    pub fn flag(mut self, name: &str) -> Self {
        self.0.push(vec![name.to_string()]);
        self
    }
}

impl From<Tags> for Vec<Vec<String>> {
    fn from(tags: Tags) -> Vec<Vec<String>> {
        tags.0
    }
}

#[cfg(test)]
mod tests;
