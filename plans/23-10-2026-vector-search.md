# Vector search: the agent finds rules and past work by meaning

## Goal

Today the `celis:past` Haiku agent reads every live rule of every project in pages. One ask can cost up
to 300k tokens and 8-10 minutes, and the cost grows with the store. celis gets a local embedding model
and a `celis search` command. The main agent asks it when it needs to, writes the query and judges the
results. With no model, the agent uses `celis grep`.

## Evidence

On 2026-10-23 the store had 1,025 live rule lines (231 KB, about 58k tokens). Only 456 were in scope for
this project. The past agent read them all at each ask, then searched and copied lines in more turns,
and each turn sent the full context again.

## Design

- `celis setup` downloads a model from Hugging Face into `$CELIS_HOME/models/<id>/` and writes the choice
  in `$CELIS_HOME/config.json`. On a TTY it is a wizard with a list of models; else it takes `--model`.
- The index is `$CELIS_HOME/index/<model>.bin`: a header with the model id and the dimensions, then one
  row for each record id. It stays local and is never synced.
- A search first embeds the live records that have no row, then the query, and ranks by the dot
  product. The store gives which records are live, so a dropped or replaced record never shows.
- `celis search` prints records by score with their scope, date and places. It applies no caps or
  priors beyond `--limit`. With no model, it prints a note that names `celis grep` and `celis setup`.
- The agent decides when to search, what to query and what applies.

Models (one table in the code: repo, pinned revision, sha256, dimensions, prefixes, license):

| Model                  | Download | Speed   | Note                                      | License      |
| ---------------------- | -------- | ------- | ----------------------------------------- | ------------ |
| EmbeddingGemma 300M    | ~1.2 GB  | fast    | Recommended: best quality for its size    | Gemma terms  |
| Qwen3-Embedding 0.6B   | ~1.2 GB  | slow    | strong on code and many languages         | Apache 2.0   |
| multilingual-e5-small  | ~480 MB  | fastest | lower quality                             | MIT          |
| none                   | 0        | —       | grep only                                 | —            |

## Phases

All phases ship in one release.

0. **Runtime spike.** Run each model in pure Rust (`candle`, else `tract`), because the Linux release is
   musl and `ort` has no musl build. → verify: a static binary embeds one query on mac and Linux musl,
   and matches the Python reference to a cosine above 0.99; record the binary size and the time.
1. **Spec.** CIP-02 gets "Search" and "Setup". The index format stays out of the spec. → verify: the
   change keeps "How a CIP works".
2. **`Embedding`.** Loads a model, adds its prefixes, returns a normalized vector. → verify: unit tests
   with a fake model; one local test with the real model.
3. **`celis setup`.** The wizard, `--model <id>|none` and `--list`; download, sha256 check, index
   build. → verify: tests with a fake download for each model, `none`, a bad checksum and no TTY.
4. **`Index`.** Reads, writes, adds rows by record id, rebuilds on a model change or a bad file. →
   verify: unit tests with a fake embedding.
5. **`celis search`.** `--kind`, `--scope`, `--limit`, `--json`, and the note with no model. → verify:
   integration tests with a fake embedding.
6. **Plugin.** The card and `steps.md` teach when to search and when to grep; `help.rs` has the usage.
   → verify: `crates/celis/tests/help.rs` and `make verify`.
   - **6a. Bench the main agent's own search.** Arms: A the past agent (today), B the main agent with
     grep only, C the main agent with `celis search` and grep. Haiku and Sonnet. Measure tokens, time,
     rules kept, steps skipped and how often the agent searches. This bench runs before the plan is
     done. → verify: a results table.
   - **6b. Remove the past agent** only when B and C are not worse than A.
7. **Release.** CI builds for `aarch64/x86_64-apple-darwin` and `aarch64/x86_64-unknown-linux-musl`;
   the model is not in the binary. Run the costly benches once. → verify: the release workflow passes.

## Spike result

Measured on 2026-10-23 on an M-series Mac, CPU, f32, one text at a time.

- **Runtime:** `candle` 0.11 (`candle-core`, `candle-nn`, `candle-transformers`) and `tokenizers` with
  `fancy-regex`, all pure Rust. The ONNX builds of Gemma and Qwen use `com.microsoft` operators
  (`MultiHeadAttention`, `RotaryEmbedding`), so `tract` cannot run them.
- **Builds:** the spike builds for `aarch64/x86_64-apple-darwin` and, with `cargo zigbuild`, for
  `aarch64/x86_64-unknown-linux-musl` as static binaries. It adds about 1.5-2 MB to the binary. The musl
  binaries are built, not yet run.
- **Sources (safetensors, pinned):** `google/embeddinggemma-300m` is gated, so Gemma comes from the ungated
  `unsloth/embeddinggemma-300m` (`bfa3c846`); Qwen from `Qwen/Qwen3-Embedding-0.6B` (`97b0c614`); e5 from
  `intfloat/multilingual-e5-small` (`614241f6`).
- **Match with the ONNX Runtime reference:** cosine 0.99998 or more for each model on 3 texts (English,
  Russian, short).

| Model   | Load  | Per text (8-18 tokens) | Code                                                                  |
| ------- | ----- | ---------------------- | --------------------------------------------------------------------- |
| Gemma   | 2.3 s | 40 ms                  | own encoder (~150 lines): Gemma 3 layers, no mask, mean pool, 2 dense |
| Qwen3   | <1 s  | 0.5-0.7 s              | `candle_transformers::models::qwen3`, last token                      |
| e5      | 0.7 s | 10-17 ms               | `candle_transformers::models::bert`, mean pool                        |

EmbeddingGemma stays Recommended. Its encoder embeds one text with no padding and at most 256 tokens,
so its 512-token sliding window never applies and it needs no mask.

## Bench 6a result

`evals/drift`, 400 rules, 8 tasks, 1 run each arm, 2026-10-23. `final` is v0.0.12 with the past agent;
`grepped` and `searched` are this branch, with the main agent searching itself, with no model and with
EmbeddingGemma. Broken counts the rules a task applies to, habits left out.

| Arm      | Haiku broken | Haiku old | Haiku $/task | Haiku s/task | Sonnet broken | Sonnet old | Sonnet $/task | Sonnet s/task |
| -------- | ------------ | --------- | ------------ | ------------ | ------------- | ---------- | ------------- | ------------- |
| final    | 26%          | 3%        | 0.05         | 193          | 23%           | 7%         | 0.25          | 130           |
| grepped  | 38%          | 11%       | 0.03         | 130          | 43%           | 20%        | 0.19          | 54            |
| searched | 36%          | 14%       | 0.03         | 121          | 55%           | 54%        | 0.17          | 46            |

- The main agent searched once per task in every arm (8 searches in 8 tasks) and did not search again
  later in the work. The past agent ran 106 greps per 8 tasks: it read every rule.
- Steps were not skipped: every task added its ask, but 2 Sonnet `searched` tasks.
- The main agent's search costs 25-35% less and takes 30-65% less time, but breaks more rules. Haiku's gap
  (10-12 points) is within the bench's noise of 15 points; Sonnet's (20-32 points) is not.
- So 6b does not hold: the past agent stays until a change closes the gap.

Second run, with `step start` naming a `celis search` for each step:

| Arm      | Haiku broken | Haiku old | Sonnet broken | Sonnet old |
| -------- | ------------ | --------- | ------------- | ---------- |
| final    | 26%          | 3%        | 23%           | 7%         |
| grepped  | 34%          | 25%       | 46%           | 32%        |
| searched | 39%          | 14%       | 48%           | 31%        |

- Drift tasks have one step each, so the agents still searched about once a task.
- **Recall is the cause.** A task applies 20-28 of the rules. Ranked by the agent's own query, those rules
  spread over all 400 ranks: recall is 14% in the top 20, 25% in the top 50, and 38% in the top 100. A
  rule found was mostly kept (Sonnet broke 3 of 36 found, 60 of 164 missed).
- Drift's rules are conventions for all code, such as absolute imports or module docstrings. Their words
  are not alike to the task's words, so no query of the task finds them. Only reading every rule, as the
  past agent does, finds them.

Third run, with the main agent reading `celis rules --all` at the ask, then `celis search`:

| Arm      | Haiku broken | Haiku $/task | Haiku s/task | Sonnet broken | Sonnet $/task | Sonnet s/task |
| -------- | ------------ | ------------ | ------------ | ------------- | ------------- | ------------- |
| final    | 26%          | 0.05         | 193          | 23%           | 0.25          | 130           |
| grepped  | 40%          | 0.03         | 130          | 33%           | 0.24          | 57            |
| searched | 35%          | 0.04         | 149          | 35%           | 0.23          | 63            |

- Every task read both pages of the rules; Sonnet cut a page with `head` in 4 of 8 tasks.
- Sonnet's gap fell from 20-32 points to 10-12; Haiku's stays at 9-14. The cost is the same as the past
  agent's; the time is half of it on Sonnet.
- The past agent gives the main agent a short list of the rules that apply. A dump of every rule leaves
  that choice to the main agent, among its other work, and it keeps fewer of them.

Fourth run, `lean`: the past agent reads `celis rules --all` (pages of 24 KB) and finds past asks with
`celis search`; the main agent does not search.

| Arm   | Haiku broken | Haiku s/task | Sonnet broken | Sonnet $/task | Sonnet s/task | Applicable rules in the answer (Haiku / Sonnet) |
| ----- | ------------ | ------------ | ------------- | ------------- | ------------- | ----------------------------------------------- |
| final | 26%          | 193          | 23%           | 0.25          | 130           | 73 / 184 of about 200                           |
| lean  | 46%          | 142          | 31%           | 0.20          | 115           | 45 / 146 of about 200                           |

- The past agent made 5-7 calls in place of 12-15, but kept fewer of the rules that apply. The old agent
  read pages of 50 lines; the new one reads pages of about 120 lines, and Haiku picks worse from a long
  page.

## Risks

| Risk                                                         | Mitigation                                                        |
| ------------------------------------------------------------ | ----------------------------------------------------------------- |
| The agent skips the search, as Sonnet skipped steps without the past agent | The card teaches it; 6a measures it; 6b waits for the result |
| `candle` lacks a part of EmbeddingGemma, or `tract` an operator | Phase 0 finds it first; Qwen3 becomes Recommended              |
| Pure-Rust CPU inference is slower than ONNX Runtime          | Phase 0 measures it; the index is small                           |
| The Gemma repo is gated on Hugging Face                      | Phase 0 checks it; the wizard asks for `HF_TOKEN` and shows the terms |
| Cold start: model load per call, first index of the store    | `setup` builds the index with progress; later searches embed only new records |
| Users with no model get grep only                            | 6a benches both; the card says grep always works                 |
| A tampered download                                          | Pin the revision and the sha256                                   |

## Decisions

- Qwen3 and e5-small are in the list beside EmbeddingGemma; the wizard marks one Recommended.
- Models come from Hugging Face.
- With no model, the main agent uses `celis grep`.
- Targets: mac and Linux.
- The 6a bench may run before the plan is done.
- When EmbeddingGemma cannot run in pure Rust, Qwen3 is Recommended.

## Open questions

None.
