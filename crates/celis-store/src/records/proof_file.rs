//! A file of a proof.

/// A file of a proof: its place when the step closed, the sha256 of its bytes in lower-case hex, its size
/// in bytes and its media type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofFile {
    pub place: String,
    pub sha256: String,
    pub size: u64,
    pub media: String,
}

impl ProofFile {
    /// The file at `place` whose bytes hash to `sha256` and count `size`, with the media type of its
    /// extension.
    pub fn new(place: String, sha256: String, size: u64) -> ProofFile {
        let media = match ProofFile::extension(&place).as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            "mp3" => "audio/mpeg",
            "wav" => "audio/wav",
            "ogg" => "audio/ogg",
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            "pdf" => "application/pdf",
            "html" => "text/html",
            "md" => "text/markdown",
            "txt" => "text/plain",
            "json" => "application/json",
            "glb" => "model/gltf-binary",
            "gltf" => "model/gltf+json",
            _ => "application/octet-stream",
        };
        ProofFile {
            place,
            sha256,
            size,
            media: media.to_string(),
        }
    }

    /// Its Blossom URI (BUD-10), with `pubkey` as the account that keeps it.
    pub fn url(&self, pubkey: &str) -> String {
        let ext = match ProofFile::extension(&self.place).as_str() {
            "" => "bin".to_string(),
            ext => ext.to_string(),
        };
        format!("blossom:{}.{ext}?as={pubkey}&sz={}", self.sha256, self.size)
    }

    /// Its `imeta` tag (NIP-92), with `pubkey` as the account that keeps it.
    pub fn tag(&self, pubkey: &str) -> Vec<String> {
        vec![
            "imeta".to_string(),
            format!("url {}", self.url(pubkey)),
            format!("x {}", self.sha256),
            format!("m {}", self.media),
            format!("size {}", self.size),
            format!("path {}", self.place),
        ]
    }

    /// The file an `imeta` `tag` names; none when it is another tag or has no `x`.
    pub fn read(tag: &[String]) -> Option<ProofFile> {
        if tag.first()? != "imeta" {
            return None;
        }
        let field = |key: &str| {
            tag.iter()
                .find_map(|f| f.strip_prefix(key)?.strip_prefix(' '))
                .map(str::to_string)
        };
        Some(ProofFile {
            place: field("path").unwrap_or_default(),
            sha256: field("x")?,
            size: field("size").and_then(|s| s.parse().ok()).unwrap_or(0),
            media: field("m").unwrap_or_default(),
        })
    }

    /// The file the values of a tag name as a Blossom URI and a place; none when the first is no Blossom
    /// URI.
    pub fn blossom(values: &[String]) -> Option<ProofFile> {
        let blob = values.first()?.strip_prefix("blossom:")?;
        let (sha256, query) = blob.split_once('.')?;
        let size = query
            .split(['?', '&'])
            .find_map(|q| q.strip_prefix("sz="))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let place = values.get(1).cloned().unwrap_or_default();
        Some(ProofFile::new(place, sha256.to_string(), size))
    }

    /// The lower-case extension of `place`; empty when it has none.
    fn extension(place: &str) -> String {
        std::path::Path::new(place)
            .extension()
            .and_then(|e| e.to_str())
            .filter(|e| e.chars().all(|c| c.is_ascii_alphanumeric()))
            .unwrap_or("")
            .to_ascii_lowercase()
    }
}
