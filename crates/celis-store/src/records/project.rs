//! A code project: its folder, id and name.

use crate::Anchor;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A code project: its folder, and its anchor: the id every clone and worktree of it shares, and its
/// name.
#[derive(Clone)]
pub struct Project {
    pub folder: PathBuf,
    pub anchor: Anchor,
}

impl Project {
    /// The project that holds `folder`: the top of its git repo or worktree, named by the repo's first
    /// commit and its main checkout. Outside git, or with no commit, the folder's path names it.
    pub fn at(folder: &Path) -> Project {
        Project::open(folder).unwrap_or_else(|_| Project::outside(folder))
    }

    /// The project that holds `folder`, as [`Project::at`] gives it; under a `.git` where git fails, what
    /// git said, with no project (CIP-01, "Projects").
    pub fn open(folder: &Path) -> Result<Project, String> {
        let inside = folder.ancestors().any(|f| f.join(".git").exists());
        let args = [
            "rev-parse",
            "--path-format=absolute",
            "--show-toplevel",
            "--git-common-dir",
        ];
        let paths = match inside {
            true => Some(
                Project::git(folder, &args)
                    .map_err(|said| format!("git failed in {}: {said}", folder.display()))?,
            ),
            false => None,
        };
        let (top, common) = match paths.as_deref().and_then(|p| p.split_once('\n')) {
            Some((top, common)) => (Some(PathBuf::from(top)), Some(PathBuf::from(common))),
            None => (None, None),
        };
        let mut roots: Vec<String> = top
            .as_ref()
            .and_then(|_| Project::git(folder, &["rev-list", "--max-parents=0", "HEAD"]).ok())
            .map(|out| out.lines().map(str::to_string).collect())
            .unwrap_or_default();
        roots.sort();
        let folder = top.unwrap_or_else(|| folder.to_path_buf());
        let id = roots
            .into_iter()
            .next()
            .unwrap_or_else(|| folder.display().to_string());
        let main = common
            .and_then(|common| common.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| folder.clone());
        let name = main
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| main.display().to_string());
        Ok(Project {
            folder,
            anchor: Anchor { project: id, name },
        })
    }

    /// The project of `folder` outside git: its path names it.
    fn outside(folder: &Path) -> Project {
        let name = folder
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| folder.display().to_string());
        Project {
            folder: folder.to_path_buf(),
            anchor: Anchor {
                project: folder.display().to_string(),
                name,
            },
        }
    }

    /// Whether the project's folder lies in a git repo.
    pub fn in_git(&self) -> bool {
        self.folder.ancestors().any(|f| f.join(".git").exists())
    }

    /// The project a step at `place` names: outside git, a URL names it; else this project.
    pub fn of_place(&self, place: &str) -> Project {
        let inside = self.in_git();
        let url = place
            .strip_prefix("https://")
            .or_else(|| place.strip_prefix("http://"));
        let (false, Some(url)) = (inside, url) else {
            return self.clone();
        };
        let mut parts = url.split(['?', '#']).next().unwrap_or("").split('/');
        let host = parts.next().unwrap_or("");
        let host = host.rsplit('@').next().unwrap_or(host);
        let host = host.split(':').next().unwrap_or(host).to_lowercase();
        if host.is_empty() {
            return self.clone();
        }
        let figma = host == "figma.com" || host.ends_with(".figma.com");
        let key = parts.next().filter(|_| figma).and_then(|kind| {
            let key = parts.next().filter(|k| !k.is_empty())?;
            matches!(kind, "file" | "design").then_some(key)
        });
        self.with(match key {
            Some(key) => Anchor {
                project: format!("figma:{key}"),
                name: parts
                    .next()
                    .filter(|n| !n.is_empty())
                    .map_or(host, str::to_string),
            },
            None => Anchor {
                project: format!("web:{host}"),
                name: host,
            },
        })
    }

    /// The project `anchor` names, at this project's folder.
    pub fn with(&self, anchor: Anchor) -> Project {
        Project {
            folder: self.folder.clone(),
            anchor,
        }
    }

    /// What `git args` prints in `folder`, trimmed, else the last line git said when it does not run or
    /// refuses.
    pub(crate) fn git(folder: &Path, args: &[&str]) -> Result<String, String> {
        let out = Command::new("git")
            .args(args)
            .current_dir(folder)
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            let said = String::from_utf8_lossy(&out.stderr);
            return Err(said
                .lines()
                .last()
                .unwrap_or("it exited with an error")
                .trim()
                .to_string());
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }
}
