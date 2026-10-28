# Contributing

Thank you for your help. This guide tells you how to make a change that we can merge.

## Before you start

- For a bug, open an issue with the command you ran, what it printed, and what you expected.
- For a new feature or a changed behaviour, open an issue first. We agree on the contract before the code.

## The layout

- `crates/` is one Cargo workspace: `celis` (the command line), `celis-patterns` (patterns),
  `celis-trails` (trails, steps and notes), `celis-store` (the store, projects and ids) and `celis-nostr`
  (the account, signed notes, encryption and the relay client).
- `spec/` holds the contract, as numbered CIPs. `spec/fixtures/` holds what two ends must match.
- `plugins/celis/` is the plugin: one manifest per agent, and the same hooks. Gemini CLI installs from the
  repo root, so `gemini-extension.json` and `hooks/` are there. Antigravity does too, so
  `.antigravity-plugin/` and `hooks.json` are there.

## Change the spec first

A change of behaviour starts in `spec/`. Follow the rules of
[`spec/README.md`](spec/README.md), "How a CIP works".

## Write the code

The conventions are in [`AGENTS.md`](AGENTS.md). They apply to people and agents alike. The most important:

- Write a failing test first, then make it pass.
- A test proves behaviour through the public surface. A test of a spec rule cites its section:
  ``(`spec/03-steps.md`, "Closing")``.
- A name is a noun for what a thing is. One primary type per file.
- Docstrings on types and members only. No inline comments.

## Verify

```bash
make verify            # fmt, clippy -D warnings, and the tests — what CI runs
make fmt               # write the formatting that verify checks
make install-cli       # build `celis` from this checkout into ~/.local/bin
make release V=1.2.3   # open the release PR of 1.2.3, for maintainers
```

To try the plugin from this checkout for one session, run `claude --plugin-dir plugins/celis` or
`cursor-agent --plugin-dir plugins/celis`.

## Open a pull request

- Branch from `dev`, open the pull request into `dev`, and keep one change in one pull request. A fix goes
  to `dev` too. `main` takes only the release PR from `dev`.
- Write the commit title as `<type>: <what changed>`, where the type is `feat`, `fix`, `docs`, `refactor`,
  `test` or `chore`.
- Say in the description which CIP the change follows, and how you checked it.

By contributing, you agree that your work is licensed under the [MIT License](LICENSE).

## Release

Only maintainers release. Agents install the plugin from `main`, so `main` moves only by a release: the
"Release source" check fails any other PR into `main`.

1. On an up-to-date `dev`, run `make release V=1.2.3`. It checks the version and your checkout, commits the
   version on `dev`, and opens the release PR from `dev` into `main`. If it stops, its message tells you what
   to fix.
2. Wait for the checks, including "Release builds", then merge the PR with a merge commit. Do not squash it,
   because a squash makes `main` and `dev` differ.
3. The merge builds `celis` from the merge commit, tags it `v1.2.3`, publishes the GitHub release, and
   installs it on clean runners. Agents see the new plugin version when you merge; the release follows a few
   minutes later.

If the Release run fails before it publishes, re-run it from the Actions tab. If it fails after it publishes,
fix the problem in a new PR into `dev` and release the next version.
