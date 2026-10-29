//! `celis cell import`: keep a cell from a folder or a URL.

use crate::command::{Args, Command};
use crate::failure::Failure;
use celis_patterns::{Export, PatternStore};
use celis_store::records::id::short;
use celis_store::Refusal;
use std::io::Write;

impl Command {
    /// Reads the cell of the folder or URL `args` names and prints its patterns; with `--yes`, keeps its
    /// records, and each blob whose bytes hash to its name. Writes no mask.
    pub(crate) fn cell_import(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let source = args.id("cell import")?;
        let text = self
            .fetch(source, Export::RECORDS)
            .ok_or_else(|| Refusal::Invalid(format!("{source} has no {}", Export::RECORDS)))?;
        let export = Export::read(&String::from_utf8_lossy(&text))?;
        let trails = self.trails()?;
        let cell = &export.cell;
        let cells = trails.patterns();
        let named = cells.cells().named(&cell.name);
        if named.iter().any(|c| c.author != cell.author) {
            return Err(Refusal::CellTaken(cell.name.clone()).into());
        }
        let npub = export.membership.pubkey.npub();
        let count = crate::text::line::plural(export.patterns.len(), "pattern");
        let mut lines = vec![format!("{} {} · {count}", cell.name, &npub[5..13])];
        for (note, record) in &export.patterns {
            lines.push(format!("  {} {}", short(&note.id), record.title()));
        }
        if !args.yes {
            lines.push(format!(
                "celis: read the patterns above; keep them with celis cell import {source} --yes"
            ));
            return crate::command::put(out, &lines.join("\n"));
        }
        for file in export.files() {
            if let Some(bytes) = self.fetch(source, &Export::blob(file)) {
                trails.hold(file, &bytes)?;
            }
        }
        let records = export.patterns.iter().map(|(note, _)| note);
        for note in records.chain([&export.membership]) {
            trails.put(note)?;
        }
        lines.push(format!(
            "celis: kept {} off; use it here with celis cell on {}",
            cell.name, cell.name
        ));
        crate::command::put(out, &lines.join("\n"))
    }

    /// The bytes of the file `name` of the cell at `source`: a URL, fetched with curl, else a folder
    /// from the working folder; none when it has none.
    fn fetch(&self, source: &str, name: &str) -> Option<Vec<u8>> {
        if !source.contains("://") {
            return std::fs::read(self.folder.join(source).join(name)).ok();
        }
        let base = source
            .trim_end_matches(Export::RECORDS)
            .trim_end_matches(Export::PAGE);
        let url = format!("{}/{name}", base.trim_end_matches('/'));
        let fetched = std::process::Command::new("curl")
            .args(["-fsSL", &url])
            .output()
            .ok()?;
        fetched.status.success().then_some(fetched.stdout)
    }
}
