//! A model and the vectors of the texts it has read.

use super::encoder::Encoder;
use super::model::Model;
use super::vectors::Vectors;
use anyhow::Result;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The chosen model, loaded, with the file of the vectors it has read.
pub struct Index {
    encoder: Encoder,
    vectors: Vectors,
    file: PathBuf,
}

impl Index {
    /// The model `model` set up in `home`, and its vectors.
    pub fn open(model: &'static Model, home: &Path) -> Result<Index> {
        let file = home.join("index").join(format!("{}.bin", model.id));
        Ok(Index {
            encoder: Encoder::load(model, &model.folder(home))?,
            vectors: Vectors::open(&file, model.id, model.dimensions),
            file,
        })
    }

    /// Reads each of `texts` that has no vector yet, drops the vectors of other texts, and saves; with
    /// `progress`, writes the count read so far to it.
    pub fn refresh(&mut self, texts: &[&str], mut progress: Option<&mut dyn Write>) -> Result<()> {
        let missing: Vec<&str> = texts
            .iter()
            .copied()
            .filter(|t| self.vectors.get(t).is_none())
            .collect();
        for (n, text) in missing.iter().enumerate() {
            self.vectors.insert(text, self.encoder.document(text)?);
            if let Some(out) = progress.as_mut() {
                let _ = write!(out, "\rcelis: indexing {}/{}", n + 1, missing.len());
            }
        }
        if let (Some(out), false) = (progress, missing.is_empty()) {
            let _ = writeln!(out);
        }
        self.vectors.keep(texts.iter().copied());
        self.vectors.save(&self.file)
    }

    /// The cosine of `query` and each text read, none for a text not read.
    pub fn alike(&self, query: &str) -> Result<impl Fn(&str) -> Option<f32> + '_> {
        let query = self.encoder.query(query)?;
        Ok(move |text: &str| {
            let vector = self.vectors.get(text)?;
            Some(query.iter().zip(vector).map(|(a, b)| a * b).sum())
        })
    }
}
