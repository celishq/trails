//! Where a pattern holds.

/// Where a pattern holds, narrowest first: while one trail of its project is active, in its project, or
/// in every project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    Trail(String),
    Project,
    Me,
}

impl Scope {
    /// The scope `text` names, as a `scope` tag or `--scope` states it; `None` when it names none.
    pub fn parse(text: &str) -> Option<Scope> {
        match text.split_once(':') {
            Some(("trail", id)) if !id.is_empty() => Some(Scope::Trail(id.to_string())),
            None if text == "project" => Some(Scope::Project),
            None if text == "me" => Some(Scope::Me),
            _ => None,
        }
    }

    /// The scope as text: `trail:<id>`, `project` or `me`.
    pub fn text(&self) -> String {
        match self {
            Scope::Trail(id) => format!("trail:{id}"),
            Scope::Project => "project".to_string(),
            Scope::Me => "me".to_string(),
        }
    }

    /// The scope as a line shows it, with a trail's id cut to its first 8 characters.
    pub fn short(&self) -> String {
        match self {
            Scope::Trail(id) => format!("trail:{}", crate::records::id::short(id)),
            scope => scope.text(),
        }
    }
}
