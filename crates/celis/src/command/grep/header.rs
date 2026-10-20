//! The header of a trail's lines in the output.

use crate::text::clock::date;
use celis_trails::records::id::short;
use celis_trails::Step;

/// The header of the lines of no trail.
pub(super) const NONE: &str = "- rules of no trail";

/// The header of a trail's lines: its id, its project, its title and the day it was made.
pub(super) struct Header {
    id: String,
    project: String,
    title: String,
    day: String,
}

impl Header {
    pub fn of(root: Step) -> Header {
        let record = root.record();
        Header {
            id: short(root.id()),
            project: record.anchor.name.clone(),
            title: record.title().to_string(),
            day: date(root.note().created_at),
        }
    }

    /// `<id, first 8> [<project name>] <title> · <yyyy-mm-dd>`.
    pub fn text(&self) -> String {
        format!(
            "{} [{}] {} · {}",
            self.id, self.project, self.title, self.day
        )
    }
}
