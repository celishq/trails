//! `celis cell export`: write a cell as a folder.

use crate::command::{Args, Command};
use crate::failure::Failure;
use crate::text::markdown;
use celis_patterns::{Example, Export, PatternStore};
use celis_store::Refusal;
use std::io::Write;

impl Command {
    /// Writes the cell of the account that `args` names to `--to`, else to `./<name>/`, as CIP-09
    /// gives it, with `cell.md`; prints the folder. Writes no record.
    pub(crate) fn cell_export(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let name = args.id("cell export")?;
        if !self.held()? {
            return Err(Refusal::NoCell(name.to_string()).into());
        }
        let trails = self.trails()?;
        let export = trails.export(name)?;
        let folder = self.folder.join(args.to.as_deref().unwrap_or(name));
        export.write(&folder, &page(&export), |file| {
            let blob = trails.blob(&file.sha256);
            let blob =
                blob.ok_or_else(|| anyhow::anyhow!("the store lacks the blob of {}", file.place))?;
            Ok(std::fs::read(blob)?)
        })?;
        crate::command::put(out, &folder.display().to_string())
    }
}

/// `cell.md`: each pattern of `export` as a section with its examples, then the credits, with the
/// author and the date of its membership.
fn page(export: &Export) -> String {
    let name = &export.cell.name;
    let mut lines = vec![
        format!("# {name}"),
        String::new(),
        format!(
            "A cell of {} from celis. Keep it with `celis cell import <this folder or its URL> --yes`, \
             then use it with `celis cell on {name}`.",
            crate::text::line::plural(export.patterns.len(), "pattern")
        ),
    ];
    for (_, record) in &export.patterns {
        let examples = record.examples.sides().filter_map(|(side, example)| {
            let link = match example {
                Example::File(file) => Export::blob(file),
                _ => String::new(),
            };
            markdown::example(side, example, &link, true)
        });
        lines.extend(markdown::section(record, None, examples.collect()));
    }
    let membership = &export.membership;
    lines.extend([
        String::new(),
        "---".to_string(),
        String::new(),
        "Made with [celis](https://celis.ai), which keeps what your agents learn as patterns. Install it \
         with `curl -fsSL https://celis.ai/install.sh | sh`, and learn more at https://celis.ai."
            .to_string(),
        String::new(),
        format!(
            "Exported by `{}` on {}.",
            membership.pubkey.npub(),
            crate::text::clock::date(membership.created_at)
        ),
    ]);
    lines.join("\n") + "\n"
}
