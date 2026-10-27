//! `celis cell export`: write a cell as a folder.

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_nostr::Note;
use celis_patterns::{Cell, Example, Pattern, PatternStore};
use celis_store::ProofFile;
use std::io::Write;
use std::path::Path;

impl Command {
    /// Writes the cell of the account that `args` names to `--to`, else to `./<name>/`: its live
    /// patterns and a new membership in `cell.jsonl`, the blobs of their files, and `cell.md`; prints
    /// the folder. Writes no record.
    pub(crate) fn cell_export(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let name = args.id("cell export")?;
        if !self.held()? {
            return Err(Failure::no_cell(name));
        }
        let trails = self.trails()?;
        let cell = Cell {
            name: name.to_string(),
            author: trails.author()?.unwrap_or_default(),
        };
        let patterns = trails.patterns();
        let every = patterns.every().iter().rev().filter(|p| p.is_live());
        let ours: Vec<&Pattern> = every
            .filter(|p| patterns.holding(p).contains(&&cell))
            .collect();
        if ours.is_empty() {
            return Err(Failure::no_cell(name));
        }
        let ids: Vec<&str> = ours.iter().map(|p| p.id()).collect();
        let membership = trails.seal(|account, now| Cell::membership(account, now, name, &ids))?;
        let folder = self.folder.join(args.to.as_deref().unwrap_or(name));
        let notes = ours.iter().map(|p| p.note).chain([&membership]);
        let lines: Result<Vec<String>, _> = notes.map(serde_json::to_string).collect();
        let lines = lines.map_err(anyhow::Error::from)?;
        let write = |path: &Path, bytes: &[u8]| -> Result<(), Failure> {
            std::fs::create_dir_all(path.parent().expect("a file of the folder"))
                .and_then(|()| std::fs::write(path, bytes))
                .map_err(|e| Failure::from(anyhow::Error::from(e)))
        };
        write(
            &folder.join("cell.jsonl"),
            (lines.join("\n") + "\n").as_bytes(),
        )?;
        for file in ours.iter().flat_map(|p| files(p)) {
            let blob = trails.blob(&file.sha256).ok_or_else(|| {
                Failure::from(anyhow::anyhow!(
                    "the store lacks the blob of {}",
                    file.place
                ))
            })?;
            let bytes = std::fs::read(blob).map_err(anyhow::Error::from)?;
            write(&folder.join(blob_path(file)), &bytes)?;
        }
        write(
            &folder.join("cell.md"),
            page(name, &ours, &membership).as_bytes(),
        )?;
        crate::command::put(out, &folder.display().to_string())
    }
}

/// The files of the examples of `pattern`.
fn files<'a>(pattern: &'a Pattern) -> impl Iterator<Item = &'a ProofFile> {
    let sides = pattern.record.examples.sides();
    sides.filter_map(|(_, example)| match example {
        Example::File(file) => Some(file),
        _ => None,
    })
}

/// The path of the blob of `file` in the folder: `blobs/<sha256>.<ext>`, with the extension of its
/// Blossom URI.
pub(crate) fn blob_path(file: &ProofFile) -> String {
    let url = file.url("");
    let named = url.trim_start_matches("blossom:");
    format!("blobs/{}", named.split('?').next().unwrap_or(named))
}

/// `cell.md`: each pattern of `ours` as a section with its examples, then the credits, with the author
/// and the date of `membership`.
fn page(name: &str, ours: &[&Pattern], membership: &Note) -> String {
    let mut lines = vec![
        format!("# {name}"),
        String::new(),
        format!(
            "A cell of {} from celis. Keep it with `celis cell import <this folder or its URL> --yes`, \
             then use it with `celis cell on {name}`.",
            crate::text::line::plural(ours.len(), "pattern")
        ),
    ];
    for pattern in ours {
        let record = &pattern.record;
        let mut text = record.content.lines();
        let heading = text.next().unwrap_or("");
        lines.extend([
            String::new(),
            format!("## {}", record.name().unwrap_or(heading)),
            String::new(),
        ]);
        lines.extend(text.map(|l| format!("{l}  ")));
        let examples: Vec<String> = record
            .examples
            .sides()
            .filter_map(|(side, example)| match example {
                Example::File(file) if file.media.starts_with("image/") => {
                    Some(format!("- {side}: ![{}]({})", file.place, blob_path(file)))
                }
                Example::File(file) => {
                    Some(format!("- {side}: [{}]({})", file.place, blob_path(file)))
                }
                Example::Url(url) => Some(format!("- {side}: <{url}>")),
                Example::Step(_) => None,
            })
            .collect();
        if !examples.is_empty() {
            lines.push(String::new());
            lines.extend(examples);
        }
    }
    let npub = membership.pubkey.npub();
    lines.extend([
        String::new(),
        "---".to_string(),
        String::new(),
        "Made with [celis](https://celis.ai), which keeps what your agents learn as patterns. Install it \
         with `curl -fsSL https://celis.ai/install.sh | sh`, and learn more at https://celis.ai."
            .to_string(),
        String::new(),
        format!(
            "Exported by `{npub}` on {}.",
            crate::text::clock::date(membership.created_at)
        ),
    ]);
    lines.join("\n") + "\n"
}
