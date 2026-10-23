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
