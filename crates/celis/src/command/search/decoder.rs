//! The layers of a Gemma 3 or Qwen3 embedding model.

use super::model::Architecture;
use candle_core::{DType, Device, Module, Result, Tensor, D};
use candle_nn::{embedding, linear_no_bias as linear, Activation, Embedding, Linear, VarBuilder};

/// The most tokens of a text a decoder reads.
pub(crate) const MAX_TOKENS: usize = 256;

/// The fields of a model's `config.json` that the decoder reads.
#[derive(serde::Deserialize)]
pub(crate) struct Config {
    head_dim: usize,
    #[serde(alias = "hidden_act")]
    hidden_activation: Activation,
    hidden_size: usize,
    intermediate_size: usize,
    num_attention_heads: usize,
    num_hidden_layers: usize,
    num_key_value_heads: usize,
    rms_norm_eps: f64,
    rope_theta: f64,
    rope_local_base_freq: Option<f64>,
    query_pre_attn_scalar: Option<usize>,
    vocab_size: usize,
    #[serde(default)]
    layer_types: Vec<String>,
}

/// A root-mean-square norm; Gemma scales by one more than its weight.
struct Norm {
    weight: Tensor,
    eps: f64,
}

impl Norm {
    fn new(dim: usize, cfg: &Config, gemma: bool, vb: VarBuilder) -> Result<Norm> {
        let weight = vb.get(dim, "weight")?;
        let weight = if gemma { (weight + 1.0)? } else { weight };
        Ok(Norm {
            weight,
            eps: cfg.rms_norm_eps,
        })
    }
}

impl Module for Norm {
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let rms = (x.sqr()?.mean_keepdim(D::Minus1)? + self.eps)?.sqrt()?;
        x.broadcast_div(&rms)?.broadcast_mul(&self.weight)
    }
}

/// One layer: attention, then the feed-forward block, each with its norms.
struct Layer {
    q: Linear,
    k: Linear,
    v: Linear,
    o: Linear,
    q_norm: Norm,
    k_norm: Norm,
    gate: Linear,
    up: Linear,
    down: Linear,
    activation: Activation,
    before_attention: Norm,
    after_attention: Option<Norm>,
    before_feed: Norm,
    after_feed: Option<Norm>,
    cos: Tensor,
    sin: Tensor,
    heads: usize,
    kv_heads: usize,
    head_dim: usize,
    scale: f64,
}

impl Layer {
    fn new(cfg: &Config, index: usize, gemma: bool, vb: VarBuilder) -> Result<Layer> {
        let (h, d) = (cfg.hidden_size, cfg.head_dim);
        let (a, m) = (vb.pp("self_attn"), vb.pp("mlp"));
        let local = cfg
            .layer_types
            .get(index)
            .is_some_and(|t| t == "sliding_attention");
        let base = match cfg.rope_local_base_freq {
            Some(freq) if local => freq,
            _ => cfg.rope_theta,
        };
        let inverse: Vec<f32> = (0..d)
            .step_by(2)
            .map(|i| 1.0 / base.powf(i as f64 / d as f64) as f32)
            .collect();
        let inverse = Tensor::from_vec(inverse, (1, d / 2), vb.device())?;
        let positions = Tensor::arange(0u32, MAX_TOKENS as u32, vb.device())?
            .to_dtype(DType::F32)?
            .reshape((MAX_TOKENS, 1))?;
        let angles = positions.matmul(&inverse)?;
        let norm = |name: &str| Norm::new(h, cfg, gemma, vb.pp(name));
        let (after_attention, before_feed, after_feed) = match gemma {
            true => (
                Some(norm("post_attention_layernorm")?),
                norm("pre_feedforward_layernorm")?,
                Some(norm("post_feedforward_layernorm")?),
            ),
            false => (None, norm("post_attention_layernorm")?, None),
        };
        Ok(Layer {
            q: linear(h, cfg.num_attention_heads * d, a.pp("q_proj"))?,
            k: linear(h, cfg.num_key_value_heads * d, a.pp("k_proj"))?,
            v: linear(h, cfg.num_key_value_heads * d, a.pp("v_proj"))?,
            o: linear(cfg.num_attention_heads * d, h, a.pp("o_proj"))?,
            q_norm: Norm::new(d, cfg, gemma, a.pp("q_norm"))?,
            k_norm: Norm::new(d, cfg, gemma, a.pp("k_norm"))?,
            gate: linear(h, cfg.intermediate_size, m.pp("gate_proj"))?,
            up: linear(h, cfg.intermediate_size, m.pp("up_proj"))?,
            down: linear(cfg.intermediate_size, h, m.pp("down_proj"))?,
            activation: cfg.hidden_activation,
            before_attention: norm("input_layernorm")?,
            after_attention,
            before_feed,
            after_feed,
            cos: angles.cos()?,
            sin: angles.sin()?,
            heads: cfg.num_attention_heads,
            kv_heads: cfg.num_key_value_heads,
            head_dim: d,
            scale: 1.0 / (cfg.query_pre_attn_scalar.unwrap_or(d) as f64).sqrt(),
        })
    }

    fn forward(&self, x: &Tensor, mask: Option<&Tensor>) -> Result<Tensor> {
        let (b, l, _) = x.dims3()?;
        let y = self.before_attention.forward(x)?;
        let heads = |t: Tensor, n: usize| t.reshape((b, l, n, self.head_dim))?.transpose(1, 2);
        let q = self
            .q_norm
            .forward(&heads(self.q.forward(&y)?, self.heads)?)?;
        let k = self
            .k_norm
            .forward(&heads(self.k.forward(&y)?, self.kv_heads)?)?;
        let v = heads(self.v.forward(&y)?, self.kv_heads)?;
        let (cos, sin) = (self.cos.narrow(0, 0, l)?, self.sin.narrow(0, 0, l)?);
        let q = candle_nn::rotary_emb::rope(&q.contiguous()?, &cos, &sin)?;
        let k = candle_nn::rotary_emb::rope(&k.contiguous()?, &cos, &sin)?;
        let groups = self.heads / self.kv_heads;
        let shape = (b, self.heads, l, self.head_dim);
        let k = Tensor::cat(&vec![&k; groups], 2)?.reshape(shape)?;
        let v = Tensor::cat(&vec![&v; groups], 2)?.reshape(shape)?;
        let weights = (q.matmul(&k.t()?)? * self.scale)?;
        let weights = match mask {
            Some(mask) => weights.broadcast_add(mask)?,
            None => weights,
        };
        let weights = candle_nn::ops::softmax_last_dim(&weights)?;
        let y = weights.matmul(&v.contiguous()?)?;
        let y = y.transpose(1, 2)?.reshape((b, l, ()))?.apply(&self.o)?;
        let y = after(&self.after_attention, y)?;
        let x = (y + x)?;
        let y = self.before_feed.forward(&x)?;
        let gated = y.apply(&self.gate)?.apply(&self.activation)?;
        let y = (gated * y.apply(&self.up)?)?.apply(&self.down)?;
        after(&self.after_feed, y)? + x
    }
}

/// `y` through `norm`, when there is one.
fn after(norm: &Option<Norm>, y: Tensor) -> Result<Tensor> {
    match norm {
        Some(norm) => norm.forward(&y),
        None => Ok(y),
    }
}

/// A Gemma 3 or Qwen3 embedding model, that gives one vector for a text's tokens.
pub(crate) struct Decoder {
    embed: Embedding,
    layers: Vec<Layer>,
    norm: Norm,
    dense: Vec<Linear>,
    architecture: Architecture,
    hidden: usize,
}

impl Decoder {
    /// The model of `cfg` from the weights `vb`, and Gemma's dense layers from `dense`.
    pub fn new(
        cfg: &Config,
        architecture: Architecture,
        vb: VarBuilder,
        dense: &[VarBuilder],
    ) -> Result<Decoder> {
        let gemma = architecture == Architecture::Gemma;
        let layers = (0..cfg.num_hidden_layers)
            .map(|i| Layer::new(cfg, i, gemma, vb.pp("layers").pp(i)))
            .collect::<Result<_>>()?;
        let h = cfg.hidden_size;
        let dense = match dense {
            [up, down] => vec![
                linear(h, 4 * h, up.pp("linear"))?,
                linear(4 * h, h, down.pp("linear"))?,
            ],
            _ => vec![],
        };
        Ok(Decoder {
            embed: embedding(cfg.vocab_size, h, vb.pp("embed_tokens"))?,
            layers,
            norm: Norm::new(h, cfg, gemma, vb.pp("norm"))?,
            dense,
            architecture,
            hidden: h,
        })
    }

    /// The vector of the tokens `ids`, not normalized.
    pub fn vector(&self, ids: &[u32], device: &Device) -> Result<Vec<f32>> {
        let ids = Tensor::new(ids, device)?.unsqueeze(0)?;
        let l = ids.dim(1)?;
        let mut x = self.embed.forward(&ids)?;
        let mut mask = None;
        match self.architecture {
            Architecture::Gemma => x = (x * (self.hidden as f64).sqrt())?,
            _ => mask = Some(causal(l, device)?),
        }
        for layer in &self.layers {
            x = layer.forward(&x, mask.as_ref())?;
        }
        let x = self.norm.forward(&x)?;
        let mut pooled = match self.architecture {
            Architecture::Gemma => x.mean(1)?,
            _ => x.narrow(1, l - 1, 1)?.squeeze(1)?,
        };
        for layer in &self.dense {
            pooled = pooled.apply(layer)?;
        }
        pooled.squeeze(0)?.to_vec1()
    }
}

/// The mask that lets each of `l` tokens see only itself and the tokens before it.
fn causal(l: usize, device: &Device) -> Result<Tensor> {
    let mask: Vec<f32> = (0..l)
        .flat_map(|i| (0..l).map(move |j| if j > i { f32::NEG_INFINITY } else { 0.0 }))
        .collect();
    Tensor::from_vec(mask, (l, l), device)
}
