//! A model loaded from its folder, that turns texts into vectors.

use super::gemma::{Config, Gemma, MAX_TOKENS};
use super::model::{Architecture, Model};
use anyhow::{anyhow, Result};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{self, BertModel};
use std::path::Path;
use tokenizers::Tokenizer;

/// The layers of a loaded model.
enum Network {
    Gemma(Gemma),
    Bert(BertModel),
}

/// A model loaded from its folder, with its tokenizer.
pub(crate) struct Encoder {
    model: &'static Model,
    tokenizer: Tokenizer,
    network: Network,
    device: Device,
}

impl Encoder {
    /// The model `model` from its files in `folder`.
    pub fn load(model: &'static Model, folder: &Path) -> Result<Encoder> {
        let device = Device::Cpu;
        let tokenizer =
            Tokenizer::from_file(folder.join("tokenizer.json")).map_err(|e| anyhow!(e))?;
        let config = std::fs::read_to_string(folder.join("config.json"))?;
        let weights = |file: &str| -> Result<VarBuilder<'static>> {
            let files = [folder.join(file)];
            Ok(unsafe { VarBuilder::from_mmaped_safetensors(&files, DType::F32, &device)? })
        };
        let network = match model.architecture {
            Architecture::Bert => Network::Bert(BertModel::load(
                weights("model.safetensors")?,
                &serde_json::from_str::<bert::Config>(&config)?,
            )?),
            Architecture::Gemma => Network::Gemma(Gemma::new(
                &serde_json::from_str::<Config>(&config)?,
                weights("model.safetensors")?,
                weights("2_Dense/model.safetensors")?,
                weights("3_Dense/model.safetensors")?,
            )?),
        };
        Ok(Encoder {
            model,
            tokenizer,
            network,
            device,
        })
    }

    /// The unit vector of a search text.
    pub fn query(&self, text: &str) -> Result<Vec<f32>> {
        self.vector(&format!("{}{text}", self.model.query))
    }

    /// The unit vector of a record's text.
    pub fn document(&self, text: &str) -> Result<Vec<f32>> {
        self.vector(&format!("{}{text}", self.model.document))
    }

    fn vector(&self, text: &str) -> Result<Vec<f32>> {
        let encoding = self.tokenizer.encode(text, true).map_err(|e| anyhow!(e))?;
        let ids = &encoding.get_ids()[..encoding.len().min(MAX_TOKENS)];
        let mut vector = match &self.network {
            Network::Gemma(gemma) => gemma.vector(ids, &self.device)?,
            Network::Bert(bert) => {
                let ids = Tensor::new(ids, &self.device)?.unsqueeze(0)?;
                let states = bert.forward(&ids, &ids.zeros_like()?, None)?;
                states.mean(1)?.squeeze(0)?.to_vec1()?
            }
        };
        vector.truncate(self.model.dimensions);
        let length = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        Ok(vector.into_iter().map(|x| x / length).collect())
    }
}

#[cfg(test)]
mod tests;
