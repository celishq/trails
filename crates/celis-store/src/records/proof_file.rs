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
        let media = media_of(&ProofFile::extension(&place)).unwrap_or(UNKNOWN);
        ProofFile {
            place,
            sha256,
            size,
            media: media.to_string(),
        }
    }

    /// It with `media`, else the media type of the first of `bytes`, when its extension gives none.
    pub fn typed(mut self, media: Option<String>, bytes: &[u8]) -> ProofFile {
        if self.media == UNKNOWN {
            let sniffed = || sniff(bytes).map(str::to_string);
            self.media = media.or_else(sniffed).unwrap_or(self.media);
        }
        self
    }

    /// Its extension: that of its place, else that of its media type, else `bin`.
    pub fn ext(&self) -> String {
        match ProofFile::extension(&self.place).as_str() {
            "" => extension_of(&self.media).unwrap_or("bin").to_string(),
            ext => ext.to_string(),
        }
    }

    /// Its Blossom URI (BUD-10), with `pubkey` as the account that keeps it.
    pub fn url(&self, pubkey: &str) -> String {
        format!(
            "blossom:{}.{}?as={pubkey}&sz={}",
            self.sha256,
            self.ext(),
            self.size
        )
    }

    /// Its `imeta` tag (NIP-92), with `pubkey` as the account that keeps it.
    pub fn tag(&self, pubkey: &str) -> Vec<String> {
        let mut tag = vec![
            "imeta".to_string(),
            format!("url {}", self.url(pubkey)),
            format!("x {}", self.sha256),
            format!("m {}", self.media),
            format!("size {}", self.size),
        ];
        if !self.place.is_empty() {
            tag.push(format!("path {}", self.place));
        }
        tag
    }

    /// Each file that a tag of `note` names: an `imeta`, or a Blossom URI with its place.
    pub fn all(note: &celis_nostr::Note) -> Vec<ProofFile> {
        let tags = note.tags.iter();
        let files =
            tags.filter_map(|t| ProofFile::read(t).or_else(|| ProofFile::blossom(t.get(1..)?)));
        files.collect()
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
        let ext = query.split('?').next().unwrap_or("");
        let file = ProofFile::new(place, sha256.to_string(), size);
        Some(file.typed(media_of(ext).map(str::to_string), &[]))
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

/// The media type of a file whose extension is unknown.
const UNKNOWN: &str = "application/octet-stream";

/// Each extension that celis knows, with its media type.
const KINDS: &[(&str, &str)] = &[
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
    ("svg", "image/svg+xml"),
    ("mp3", "audio/mpeg"),
    ("wav", "audio/wav"),
    ("ogg", "audio/ogg"),
    ("mp4", "video/mp4"),
    ("webm", "video/webm"),
    ("pdf", "application/pdf"),
    ("html", "text/html"),
    ("md", "text/markdown"),
    ("txt", "text/plain"),
    ("json", "application/json"),
    ("glb", "model/gltf-binary"),
    ("gltf", "model/gltf+json"),
];

/// The media type of the extension `ext`.
fn media_of(ext: &str) -> Option<&'static str> {
    KINDS.iter().find(|(e, _)| *e == ext).map(|(_, m)| *m)
}

/// The extension of the media type `media`.
fn extension_of(media: &str) -> Option<&'static str> {
    KINDS.iter().find(|(_, m)| *m == media).map(|(e, _)| *e)
}

/// The media type that the first of `bytes` show.
fn sniff(bytes: &[u8]) -> Option<&'static str> {
    let starts: [(&[u8], &str); 4] = [
        (b"\x89PNG", "image/png"),
        (b"\xff\xd8\xff", "image/jpeg"),
        (b"GIF8", "image/gif"),
        (b"%PDF", "application/pdf"),
    ];
    let webp = bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP";
    let found = starts.iter().find(|(start, _)| bytes.starts_with(start));
    found
        .map(|(_, media)| *media)
        .or(webp.then_some("image/webp"))
}
