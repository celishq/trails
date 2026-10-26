//! A topic that groups patterns.

/// A topic a pattern is in, as lower-case words joined by `-`, such as `cli-help` (CIP-05, "Cells").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell(String);

impl Cell {
    /// The cell `text` names, in its normal form; `None` when it has no letter or digit.
    pub fn new(text: &str) -> Option<Cell> {
        let words = text.split(|c: char| !c.is_alphanumeric());
        let words: Vec<String> = words
            .filter(|w| !w.is_empty())
            .map(str::to_lowercase)
            .collect();
        (!words.is_empty()).then(|| Cell(words.join("-")))
    }

    /// The cells `texts` name, each once, in their order; leaving out each with no letter or digit.
    pub fn all<'a>(texts: impl IntoIterator<Item = &'a str>) -> Vec<Cell> {
        let mut cells: Vec<Cell> = Vec::new();
        for cell in texts.into_iter().filter_map(Cell::new) {
            if !cells.contains(&cell) {
                cells.push(cell);
            }
        }
        cells
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}
