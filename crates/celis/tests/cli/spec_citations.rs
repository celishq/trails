//! The spec sections that the code cites.

use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sources(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() && !path.ends_with("target") {
            sources(&path, found);
        } else if path.extension().is_some_and(|e| e == "rs") {
            found.push(path);
        }
    }
}

/// Each spec file and section that the code cites, with its comment lines joined.
fn citations() -> Vec<(PathBuf, String, String)> {
    let mut files = Vec::new();
    sources(&root().join("crates"), &mut files);
    let mut found = Vec::new();
    for path in files {
        let text = fs::read_to_string(&path).unwrap();
        let joined = text
            .lines()
            .map(|l| {
                l.trim_start()
                    .trim_start_matches("//!")
                    .trim_start_matches("///")
            })
            .collect::<Vec<_>>()
            .join(" ");
        for (at, _) in joined.match_indices("(`spec/") {
            let rest = &joined[at + 2..];
            let Some((spec, rest)) = rest.split_once('`') else {
                continue;
            };
            let Some(rest) = rest.trim_start_matches(',').trim_start().strip_prefix('"') else {
                continue;
            };
            let Some((section, _)) = rest.split_once('"') else {
                continue;
            };
            let section = section.split_whitespace().collect::<Vec<_>>().join(" ");
            found.push((path.clone(), spec.to_string(), section));
        }
    }
    found
}

/// Each spec section that a docstring of the code cites is a heading of that spec file.
#[test]
fn each_cited_spec_section_is_a_heading_of_its_file() {
    let found = citations();
    assert!(found.len() > 100, "{}", found.len());
    for (path, spec, section) in found {
        let doc = fs::read_to_string(root().join(&spec))
            .unwrap_or_else(|_| panic!("{}: no {spec}", path.display()));
        assert!(
            doc.lines()
                .any(|l| l.trim_start_matches('#').trim() == section && l.starts_with('#')),
            "{}: {spec} has no section {section:?}",
            path.display()
        );
    }
}
