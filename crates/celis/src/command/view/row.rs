//! One line of the view.

use super::kind::Kind;

/// One line of the view: the day of its record, its trail, `None` for a rule with no trail, its
/// project, its kind, its record's id, its fields, and its text.
#[derive(Clone)]
pub(crate) struct Row {
    pub trail: Option<String>,
    pub kind: Kind,
    pub day: String,
    pub project: String,
    pub id: String,
    pub fields: Vec<String>,
    pub text: String,
}

impl Row {
    /// `<day> <trail | -> [<project>] <kind> <head> <text>`, the line `grep` searches.
    pub fn line(&self) -> String {
        let trail = self.trail.as_deref().unwrap_or("-");
        format!("{} {trail} [{}] {}", self.day, self.project, self.rest())
    }

    /// The line as the output shows it under its trail: from its kind on, after its project when it
    /// has no trail, and without the title of a trail.
    pub fn shown(&self) -> String {
        match (&self.trail, self.kind) {
            (_, Kind::Trail) => format!("{} {}", self.kind.text(), self.head()),
            (Some(_), _) => self.rest(),
            (None, _) => format!("[{}] {}", self.project, self.rest()),
        }
    }

    /// The id, then each field.
    fn head(&self) -> String {
        let words = std::iter::once(&self.id).chain(&self.fields);
        words.cloned().collect::<Vec<_>>().join(" ")
    }

    /// `<kind> <head> <text>`.
    fn rest(&self) -> String {
        format!("{} {} {}", self.kind.text(), self.head(), self.text)
    }
}
