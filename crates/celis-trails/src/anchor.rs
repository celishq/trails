//! What a decision, trail or step is on.

/// What a decision, trail or step is on: the project of this id, and the name it goes by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anchor {
    pub project: String,
    pub name: String,
}

impl Anchor {
    /// The `I` value a record carries: `project:<id>`.
    pub fn id(&self) -> String {
        format!("project:{}", self.project)
    }

    /// The `K` value a record carries: `project`.
    pub fn kind(&self) -> &'static str {
        "project"
    }

    /// The `I`, `K`, `i`, `k` and `project` tags a record on this anchor carries, in that order.
    pub fn tags(&self) -> Vec<Vec<String>> {
        let id = self.id();
        let kind = self.kind();
        let tag = |name: &str, value: &str| vec![name.to_string(), value.to_string()];
        vec![
            tag("I", &id),
            tag("K", kind),
            tag("i", &id),
            tag("k", kind),
            tag("project", &self.name),
        ]
    }
}
