//! A pattern as a section of a Markdown page.

use celis_patterns::{Example, PatternRecord};
use celis_store::ProofFile;

/// The section of `record`: its Name, else its first line, as a heading, then `meta`, the rest of its
/// text, and `tail`.
pub(crate) fn section(
    record: &PatternRecord,
    meta: Option<String>,
    tail: Vec<String>,
) -> Vec<String> {
    let mut text = record.content.lines();
    let heading = text.next().unwrap_or("");
    let mut lines = vec![
        String::new(),
        format!("## {}", record.name().unwrap_or(heading)),
        String::new(),
    ];
    if let Some(meta) = meta {
        lines.extend([meta, String::new()]);
    }
    let rest: Vec<String> = text.map(|l| format!("{l}  ")).collect();
    if !rest.is_empty() {
        lines.extend(rest);
        lines.push(String::new());
    }
    lines.extend(tail);
    lines
}

/// The line of a file or URL example on `side`, a file linked at `link`; none for a step.
pub(crate) fn example(side: &str, example: &Example, link: &str) -> Option<String> {
    match example {
        Example::File(file) => Some(format!("- {side}: {}", file_link(file, link))),
        Example::Url(url) => Some(format!("- {side}: <{url}>")),
        Example::Step(_) => None,
    }
}

/// A file as a link to `link`, or as a picture of it when it is an image.
pub(crate) fn file_link(file: &ProofFile, link: &str) -> String {
    let bang = if file.media.starts_with("image/") {
        "!"
    } else {
        ""
    };
    format!("{bang}[{}]({link})", file.place)
}
