//! The details of a tree row.

/// The details of a tree row, each in its own column: its project, its steps, its rules, its worked
/// time and its id.
#[derive(Default)]
pub(super) struct Cells {
    pub project: Option<String>,
    pub steps: Option<String>,
    pub rules: Option<String>,
    pub time: Option<String>,
    pub id: Option<String>,
}

/// The count of a row's columns.
pub(super) const COLUMNS: usize = 5;
/// The column of a row's steps.
pub(super) const STEPS: usize = 1;
/// The column of a row's worked time.
pub(super) const TIME: usize = 3;
/// The column of a row's id.
pub(super) const ID: usize = 4;

impl Cells {
    /// The text of each column, from the project to the id.
    pub fn columns(&self) -> [Option<&str>; COLUMNS] {
        [
            &self.project,
            &self.steps,
            &self.rules,
            &self.time,
            &self.id,
        ]
        .map(|c| c.as_deref())
    }
}
