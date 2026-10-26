//! Where a pattern holds.

/// Where a pattern holds: in its project, or in every project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    Project,
    Me,
}

impl Scope {
    /// The scope `text` names, as a `scope` tag or `--scope` states it; `None` when it names none.
    pub fn parse(text: &str) -> Option<Scope> {
        match text {
            "project" => Some(Scope::Project),
            "me" => Some(Scope::Me),
            _ => None,
        }
    }

    /// The scope as text: `project` or `me`.
    pub fn text(&self) -> String {
        match self {
            Scope::Project => "project".to_string(),
            Scope::Me => "me".to_string(),
        }
    }
}
