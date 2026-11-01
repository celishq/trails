//! Where the bytes of a file to keep come from.

use crate::{Place, Refusal};
use base64::Engine;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Where the bytes of a file to keep come from: a file of the working folder, stdin, or a `data:` URI
/// (`spec/02-cli.md`, "Commands").
pub enum Source {
    File { at: String, path: PathBuf },
    Stdin,
    Data(String),
}

impl Source {
    /// The source `value` names: `-` for stdin, a `data:` URI, else the file at `value` in `folder`.
    pub fn of(value: &str, folder: &Path) -> Source {
        match value {
            "-" => Source::Stdin,
            _ if value.starts_with("data:") => Source::Data(value.to_string()),
            _ => Source::File {
                at: value.to_string(),
                path: folder.join(value),
            },
        }
    }

    /// Whether it is a file that is not there.
    pub fn is_missing(&self) -> bool {
        matches!(self, Source::File { path, .. } if !path.is_file())
    }

    /// Its place from the project's top folder `top`, with `cwd` as the working folder; empty for stdin
    /// or a `data:` URI.
    pub fn place(&self, top: &Path, cwd: &Path) -> String {
        match self {
            Source::File { at, .. } => Place::resolve(at, top, cwd),
            _ => String::new(),
        }
    }

    /// Its bytes, and the media type its `data:` URI gives.
    pub fn read(&self) -> Result<(Vec<u8>, Option<String>), Refusal> {
        let usage = |text: &str| Refusal::Usage(text.to_string());
        let (bytes, media) = match self {
            Source::File { path, .. } => {
                let bytes = std::fs::read(path)
                    .map_err(|_| Refusal::Usage(format!("no file {}", path.display())))?;
                (bytes, None)
            }
            Source::Stdin => {
                let mut bytes = Vec::new();
                std::io::stdin()
                    .read_to_end(&mut bytes)
                    .map_err(|_| usage("stdin could not be read"))?;
                (bytes, None)
            }
            Source::Data(uri) => {
                let (head, data) = uri["data:".len()..]
                    .split_once(',')
                    .ok_or_else(|| usage("a data: URI needs a comma before its data"))?;
                let media = head.strip_suffix(";base64").ok_or_else(|| {
                    usage("a data: URI needs ;base64, as in data:image/png;base64,...")
                })?;
                let data: String = data.chars().filter(|c| !c.is_ascii_whitespace()).collect();
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(data)
                    .map_err(|e| Refusal::Usage(format!("the data: URI is not base64: {e}")))?;
                let media = media.split(';').next().filter(|m| !m.is_empty());
                (bytes, media.map(str::to_string))
            }
        };
        match bytes.is_empty() {
            true => Err(usage("the file has no bytes; pipe them to stdin for -")),
            false => Ok((bytes, media)),
        }
    }
}
