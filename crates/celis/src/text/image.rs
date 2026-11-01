//! Images drawn inline in a terminal, with the iTerm2 image protocol.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;

/// The escape sequence that draws `bytes` inline under the name `name`, in iTerm2 and WezTerm.
pub(crate) fn inline(name: &str, bytes: &[u8]) -> String {
    format!(
        "\x1b]1337;File=name={};size={};inline=1:{}\x07",
        STANDARD.encode(name),
        bytes.len(),
        STANDARD.encode(bytes)
    )
}

/// Whether a terminal draws inline images: stdout is one, and `program` is its `TERM_PROGRAM`.
pub(crate) fn drawn(terminal: bool, program: Option<&str>) -> bool {
    terminal && matches!(program, Some("iTerm.app" | "WezTerm"))
}

#[cfg(test)]
mod tests;
