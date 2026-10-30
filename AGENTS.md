# celis trails — repo conventions

## Layout

- `DESIGN.md` holds the design decisions. Before you change the design, read it, and change it first.
- `crates/` is one Cargo workspace. Run cargo from there. `celis-trails` uses `celis-patterns`, and both
  use `celis-store`. `celis-patterns` never uses `celis-trails`.
- `spec/` holds the contract, as CIPs. `spec/fixtures/` holds what two ends must match. Before you change
  `spec/`, read and follow `spec/README.md`, "How a CIP works".
- `plugins/celis/` is the plugin: one manifest per agent, and the same hooks. Gemini CLI installs from the
  repo root, so `gemini-extension.json` and `hooks/` are there. Antigravity does too, so
  `.antigravity-plugin/` and `hooks.json` are there. `plugins/celis-ui/` is a Claude Code mod that shows
  Celis in the status line; `claude plugin test plugins/celis-ui` runs its tests. The card's command lines
  match the usage in `crates/celis/src/help.rs`; `crates/celis/tests/cli/help.rs` checks it.
- `crates/celis/src/command` groups code by feature: `pattern/`, `step/`, `trail/`, `cell/`, `stats/`,
  `grep/`, `hook/` and `setup/`. A feature's root file holds its command and names its parts. Only code
  that all features share goes in `terminal/`.
- Workflows, the Makefile and scripts run a script as `sh scripts/<name>.sh`.
- Data from a real celis store never goes into the repo.
- Try a change that reads or rewrites the real celis store on a copy of it in a scratch folder, with
  `CELIS_HOME` set to the copy. Leave the real store for the person.
- A value that only `celis` shows, such as a grade, a tally or a period, is worked out in `crates/celis`;
  `celis-trails` holds only what the CIPs define.
- In `command/grep/`, each type has its own file (`view.rs`, `found.rs`, `view/row.rs`, `view/header.rs`,
  `view/kind.rs`), and a line is built from the fields of a Row or a Header, never parsed from a formatted
  line.

## Writing

- Say it once, in the fewest words that keep the meaning. Short sentences, never fragments.
- A name is a noun for what a thing is. No `-er`/`-or` agent names (`Manager`, `Handler`).
- One primary type per file; the filename matches it.
- The record that guides later work is a pattern. It has no `binds` label: a reader reads an old `binds`
  record as a plain pattern. A pattern is `pattern` in `celis.pattern`, and a reader reads `decision` in
  `celis.decision` as it.
- Test a change of the card's wording or of the pattern form with the evals in `evals/`, and keep the form
  that scores best.
- Text that agents read uses as few terms as it can, and says the rest in plain words.
- The card and the plugin agents teach the agent the method and why each move matters. They do not guard
  against an agent that lies.
- The headline is 'Teach your AI once, not every day', with 'Celis turns your corrections into patterns
  that every agent keeps.', on the site, the README and posts. Manifests, marketplace files and GitHub
  About say 'Celis: your team's know-how, in every AI agent.'

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
- An eval case gets a skill from outside the repo as a plugin path in its plugins list. Check the run's init
  skills list before a full run.

## Verify

```bash
make verify   # fmt, clippy -D warnings, and the tests
```

A release follows the Release section of `CONTRIBUTING.md`. The repo keeps `crates/celis` at `0.0.0`; the
release passes the version in through `CELIS_VERSION`. Open every pull request into `dev`. Only the release
PR goes from `dev` into `main`.

Before a push, run `rustup update stable` and then `make verify`: CI runs the newest stable clippy, and its
new lints fail the build.

Before each bench or eval run, tell the person which runs, on which models and at what cost, and wait for
their go. Leave benches, evals and the release to the end of a plan.
