//! A trail as a person reads it.

use crate::Step;

/// A trail as a person reads it: its head, and its parts. A session's first ask holds the trail's own
/// text, so it reads as the head's own work, and its steps as the trail's first parts.
#[derive(Clone, Copy)]
pub struct Trail<'a> {
    head: Step<'a>,
}

impl<'a> Trail<'a> {
    pub fn new(head: Step<'a>) -> Trail<'a> {
        Trail { head }
    }

    pub fn head(self) -> Step<'a> {
        self.head
    }

    /// The first ask, when it holds the text of the head and the head is a trail.
    pub fn first(self) -> Option<Step<'a>> {
        let head = self.head;
        let first = *head.children().first().filter(|_| head.is_trail())?;
        (first.record().content == head.record().content).then_some(first)
    }

    /// The step that holds the trail's own work: its first ask, else its head.
    pub fn work(self) -> Step<'a> {
        self.first().unwrap_or(self.head)
    }

    /// The steps under the head, with the first ask's steps in its place.
    pub fn parts(self) -> Vec<Step<'a>> {
        let mut parts = self.head.children();
        if let Some(first) = self.first() {
            parts.splice(0..1, first.children());
        }
        parts
    }
}
