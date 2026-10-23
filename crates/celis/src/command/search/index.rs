//! The vectors of the texts a model has read, kept in a file.

use anyhow::Result;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// The vector of each text one model has read, by the hash of the text.
pub(crate) struct Index {
    model: String,
    dimensions: usize,
    rows: HashMap<u64, Vec<f32>>,
}

impl Index {
    /// The index in `file` of `model`; empty when the file is missing, bad, or of another model.
    pub fn open(file: &Path, model: &str, dimensions: usize) -> Index {
        let mut index = Index {
            model: model.to_string(),
            dimensions,
            rows: HashMap::new(),
        };
        let bytes = std::fs::read(file).unwrap_or_default();
        let head = index.head();
        let Some(body) = bytes.strip_prefix(head.as_bytes()) else {
            return index;
        };
        let size = 8 + 4 * dimensions;
        if body.len() % size != 0 {
            return index;
        }
        for row in body.chunks(size) {
            let key = u64::from_le_bytes(row[..8].try_into().expect("8 bytes"));
            let vector = row[8..]
                .chunks(4)
                .map(|b| f32::from_le_bytes(b.try_into().expect("4 bytes")))
                .collect();
            index.rows.insert(key, vector);
        }
        index
    }

    /// Writes the index to `file`, whole or not at all.
    pub fn save(&self, file: &Path) -> Result<()> {
        let mut bytes = self.head().into_bytes();
        for (key, vector) in &self.rows {
            bytes.extend(key.to_le_bytes());
            bytes.extend(vector.iter().flat_map(|x| x.to_le_bytes()));
        }
        if let Some(folder) = file.parent() {
            std::fs::create_dir_all(folder)?;
        }
        let part = file.with_extension("part");
        std::fs::write(&part, bytes)?;
        std::fs::rename(part, file)?;
        Ok(())
    }

    pub fn get(&self, text: &str) -> Option<&[f32]> {
        self.rows.get(&key(text)).map(Vec::as_slice)
    }

    pub fn insert(&mut self, text: &str, vector: Vec<f32>) {
        self.rows.insert(key(text), vector);
    }

    /// Keeps only the vectors of `texts`.
    pub fn keep<'a>(&mut self, texts: impl IntoIterator<Item = &'a str>) {
        let kept: std::collections::HashSet<u64> = texts.into_iter().map(key).collect();
        self.rows.retain(|k, _| kept.contains(k));
    }

    fn head(&self) -> String {
        format!("celis-index 1 {} {}\n", self.model, self.dimensions)
    }
}

/// The first 8 bytes of the sha256 of `text`.
fn key(text: &str) -> u64 {
    let hash = Sha256::digest(text.as_bytes());
    u64::from_le_bytes(hash[..8].try_into().expect("8 bytes"))
}

#[cfg(test)]
mod tests;
