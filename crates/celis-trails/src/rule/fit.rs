//! How narrowly a rule's trigger fits a call.

/// How narrowly a rule's trigger fits a call, narrowest first: it names the file, it is a glob with
/// `content` or a `command` or `tool`, or it is a glob with no `content`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fit {
    Named,
    Pattern,
    Glob,
}
