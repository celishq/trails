//! The layers of the EmbeddingGemma model.

use candle_core::{DType, Device, Module, Result, Tensor, D};
use candle_nn::{embedding, linear_no_bias as linear, Activation, Embedding, Linear, VarBuilder};

/// The most tokens of a text the model reads.
pub(crate) const MAX_TOKENS: usize = 256;

/// The fields of the model's `config.json` that it reads.
#[derive(serde::Deserialize)]
pub(crate) struct Config {
    head_dim: usize,
    hidden_activation: Activation,
    hidden_size: usize,
    intermediate_size: usize,
    num_attention_heads: usize,
    num_hidden_layers: usize,
    num_key_value_heads: usize,
    rms_norm_eps: f64,
    rope_theta: f64,
    rope_local_base_freq: f64,
    query_pre_attn_scalar: usize,
    vocab_size: usize,
    layer_types: Vec<String>,
}

/// A root-mean-square norm that scales by one more than its weight.
struct Norm {
    weight: Tensor,
    eps: f64,
}

impl Norm {
    fn new(dim: usize, cfg: &Config, vb: VarBuilder) -> Result<Norm> {
        Ok(Norm {
            weight: (vb.get(dim, "weight")? + 1.0)?,
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

/// One layer: attention, then the feed-forward block, each between two norms.
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
    norms: [Norm; 4],
    cos: Tensor,
    sin: Tensor,
    heads: usize,
    kv_heads: usize,
    head_dim: usize,
    scale: f64,
}

impl Layer {
    fn new(cfg: &Config, index: usize, vb: VarBuilder) -> Result<Layer> {
        let (h, d) = (cfg.hidden_size, cfg.head_dim);
        let (a, m) = (vb.pp("self_attn"), vb.pp("mlp"));
        let base = match cfg.layer_types[index] == "sliding_attention" {
            true => cfg.rope_local_base_freq,
            false => cfg.rope_theta,
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
        let norm = |name: &str| Norm::new(h, cfg, vb.pp(name));
        Ok(Layer {
            q: linear(h, cfg.num_attention_heads * d, a.pp("q_proj"))?,
            k: linear(h, cfg.num_key_value_heads * d, a.pp("k_proj"))?,
            v: linear(h, cfg.num_key_value_heads * d, a.pp("v_proj"))?,
            o: linear(cfg.num_attention_heads * d, h, a.pp("o_proj"))?,
            q_norm: Norm::new(d, cfg, a.pp("q_norm"))?,
            k_norm: Norm::new(d, cfg, a.pp("k_norm"))?,
            gate: linear(h, cfg.intermediate_size, m.pp("gate_proj"))?,
            up: linear(h, cfg.intermediate_size, m.pp("up_proj"))?,
            down: linear(cfg.intermediate_size, h, m.pp("down_proj"))?,
            activation: cfg.hidden_activation,
            norms: [
                norm("input_layernorm")?,
                norm("post_attention_layernorm")?,
                norm("pre_feedforward_layernorm")?,
                norm("post_feedforward_layernorm")?,
            ],
            cos: angles.cos()?,
            sin: angles.sin()?,
            heads: cfg.num_attention_heads,
            kv_heads: cfg.num_key_value_heads,
            head_dim: d,
            scale: 1.0 / (cfg.query_pre_attn_scalar as f64).sqrt(),
        })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let (b, l, _) = x.dims3()?;
        let y = self.norms[0].forward(x)?;
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
        let weights = candle_nn::ops::softmax_last_dim(&weights)?;
        let y = weights.matmul(&v.contiguous()?)?;
        let y = y.transpose(1, 2)?.reshape((b, l, ()))?.apply(&self.o)?;
        let x = (self.norms[1].forward(&y)? + x)?;
        let y = self.norms[2].forward(&x)?;
        let gated = y.apply(&self.gate)?.apply(&self.activation)?;
        let y = (gated * y.apply(&self.up)?)?.apply(&self.down)?;
        self.norms[3].forward(&y)? + x
    }
}

/// The EmbeddingGemma model: every token sees every other, then mean pooling and two dense layers.
pub(crate) struct Gemma {
    embed: Embedding,
    layers: Vec<Layer>,
    norm: Norm,
    dense: [Linear; 2],
    hidden: usize,
}

impl Gemma {
    /// The model of `cfg` from the weights `vb`, and its dense layers from `up` and `down`.
    pub fn new(cfg: &Config, vb: VarBuilder, up: VarBuilder, down: VarBuilder) -> Result<Gemma> {
        let h = cfg.hidden_size;
        let layers = (0..cfg.num_hidden_layers)
            .map(|i| Layer::new(cfg, i, vb.pp("layers").pp(i)))
            .collect::<Result<_>>()?;
        Ok(Gemma {
            embed: embedding(cfg.vocab_size, h, vb.pp("embed_tokens"))?,
            layers,
            norm: Norm::new(h, cfg, vb.pp("norm"))?,
            dense: [
                linear(h, 4 * h, up.pp("linear"))?,
                linear(4 * h, h, down.pp("linear"))?,
            ],
            hidden: h,
        })
    }

    /// The vector of the tokens `ids`, not normalized.
    pub fn vector(&self, ids: &[u32], device: &Device) -> Result<Vec<f32>> {
        let ids = Tensor::new(ids, device)?.unsqueeze(0)?;
        let mut x = (self.embed.forward(&ids)? * (self.hidden as f64).sqrt())?;
        for layer in &self.layers {
            x = layer.forward(&x)?;
        }
        let pooled = self.norm.forward(&x)?.mean(1)?;
        let pooled = pooled.apply(&self.dense[0])?.apply(&self.dense[1])?;
        pooled.squeeze(0)?.to_vec1()
    }
}
