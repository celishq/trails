//! The rows of a route of stats as a Markdown table.

/// A table: the name of each column, and the cells of each row.
pub(crate) struct Table {
    pub head: Vec<&'static str>,
    pub rows: Vec<Vec<String>>,
}

impl Table {
    /// It as a Markdown table under the heading `title`.
    pub fn markdown(&self, title: &str) -> String {
        let row = |cells: Vec<String>| format!("| {} |", cells.join(" | "));
        let mut lines = vec![
            format!("### {title}"),
            String::new(),
            row(self.head.iter().map(|h| h.to_string()).collect()),
            row(self.head.iter().map(|_| "---".to_string()).collect()),
        ];
        for cells in &self.rows {
            lines.push(row(cells.iter().map(|c| c.replace('|', "\\|")).collect()));
        }
        lines.join("\n") + "\n"
    }
}

#[cfg(test)]
mod tests;
