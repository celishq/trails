//! How narrowly a pattern's trigger fits a file.

/// How narrowly a pattern's trigger fits a file, narrowest first: it names the file, or it is a glob.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fit {
    Named,
    Glob,
}
