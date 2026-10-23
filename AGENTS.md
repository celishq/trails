# celis trails — repo conventions

## Layout

- `crates/` is one Cargo workspace. Run cargo from there.
- `spec/` holds the contract, as CIPs. `spec/fixtures/` holds what two ends must match. Before you change
  `spec/`, read and follow `spec/README.md`, "How a CIP works".
- `plugins/celis/` is the plugin: one manifest per agent, and the same hooks. Gemini CLI installs from the
  repo root, so `gemini-extension.json` and `hooks/` are there. Antigravity does too, so
  `.antigravity-plugin/` and `hooks.json` are there. The card's command lines match the usage in
  `crates/celis/src/help.rs`; `crates/celis/tests/help.rs` checks it.
- `crates/celis/src` groups code by feature (tree, stats, graph, report). Only code that all features
  share goes in `terminal/`.
- Workflows, the Makefile and scripts run a script as `sh scripts/<name>.sh`.
- Data from a real celis store never goes into the repo.

## Writing

- Say it once, in the fewest words that keep the meaning. Short sentences, never fragments.
- A name is a noun for what a thing is. No `-er`/`-or` agent names (`Manager`, `Handler`).
- One primary type per file; the filename matches it.
- The record that later work must keep is a rule. The wire names (`celis.decision`, `celis decide`, the
  `--json` keys) stay.

## Docstrings

- On types and members only. No inline comments.
- A docstring states what the item does, not how or why. Five lines on a type, three on a member, none
  where the name says it.
- A test of a spec rule cites its section: ``(`spec/03-steps.md`, "Section")``.
- A shell script or a workflow has one header comment that says what it does, and no comments in its
  steps.

## Tests

- Unit tests go in a sibling `tests.rs` (`foo.rs` → `foo/tests.rs`).
- A test proves behaviour through the public surface. Fakes stand in only at I/O.

## Verify

```bash
make verify   # fmt, clippy -D warnings, and the tests
```

A release follows the Release section of `CONTRIBUTING.md`. The repo keeps `crates/celis` at `0.0.0`; the
release passes the version in through `CELIS_VERSION`.
