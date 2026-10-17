//! A place: a file, a `file:line`, a folder or a URL that a step or a decision is at.

use std::path::{Component, Path, PathBuf};

/// A place as a record holds it: a path from the project's top folder, with its `:<line>` kept, or a URL
/// or an outside path as given.
#[derive(Clone, Copy)]
pub struct Place<'a>(&'a str);

impl<'a> Place<'a> {
    pub fn new(at: &'a str) -> Place<'a> {
        Place(at)
    }

    /// The file or folder, without a trailing `:<line>`.
    pub fn path(self) -> &'a str {
        match self.0.rsplit_once(':') {
            Some((path, line)) if !line.is_empty() && line.bytes().all(|b| b.is_ascii_digit()) => {
                path
            }
            _ => self.0,
        }
    }

    /// Whether the two meet: the same file or folder, whatever their lines, or one lies in the other.
    pub fn meets(self, other: Place) -> bool {
        let (a, b) = (self.path(), other.path().trim_end_matches('/'));
        a == b || inside(a, b) || inside(b, a)
    }

    /// `at`, given in the folder `cwd`, as a record holds it for the project whose top folder is `top`.
    pub fn resolve(at: &str, top: &Path, cwd: &Path) -> String {
        if at.contains("://") {
            return at.to_string();
        }
        let path = Place(at).path();
        let line = &at[path.len()..];
        let cwd = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
        let top = top.canonicalize().unwrap_or_else(|_| top.to_path_buf());
        let joined = cwd.join(path);
        let joined = joined.canonicalize().unwrap_or(joined);
        let mut full = PathBuf::new();
        for part in joined.components() {
            match part {
                Component::CurDir => {}
                Component::ParentDir => {
                    full.pop();
                }
                part => full.push(part),
            }
        }
        match full.strip_prefix(&top) {
            Ok(inner) if inner.as_os_str().is_empty() => format!(".{line}"),
            Ok(inner) => format!("{}{line}", inner.display()),
            Err(_) => at.to_string(),
        }
    }
}

/// Whether the path `inner` lies in the folder `outer`.
fn inside(inner: &str, outer: &str) -> bool {
    inner
        .strip_prefix(outer)
        .is_some_and(|rest| rest.starts_with('/'))
}
