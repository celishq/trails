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
| EmbeddingGemma 300M    | ~300 MB  | fast    | Recommended: best quality for its size    | Gemma terms  |
| Qwen3-Embedding 0.6B   | ~600 MB  | medium  | strong on code and many languages         | Apache 2.0   |
| multilingual-e5-small  | ~120 MB  | fastest | lower quality                             | MIT          |
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
