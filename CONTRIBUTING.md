# Contributing

Thank you for your help. This guide tells you how to make a change that we can merge.

## Before you start

- For a bug, open an issue with the command you ran, what it printed, and what you expected.
- For a new feature or a changed behaviour, open an issue first. We agree on the contract before the code.

## The layout

- `crates/` is one Cargo workspace: `celis` (the command line), `celis-trails` (trails, steps, decisions
  and their store) and `celis-nostr` (the account, signed notes, encryption and the relay client).
- `spec/trails/` holds the contract, as numbered CIPs. `spec/fixtures/` holds what two ends must match.
- `plugins/celis/` is the plugin: one manifest per agent, and the same hooks. Gemini CLI installs from the
  repo root, so `gemini-extension.json` and `hooks/` are there. Antigravity does too, so
  `.antigravity-plugin/` and `hooks.json` are there.

## Change the spec first

A change of behaviour starts in `spec/trails/`. Read [`spec/trails/README.md`](spec/trails/README.md) for
how a CIP works. In short:

- A CIP adds. It never changes another CIP. A new meaning for a name is a new CIP, with the next number.
- A final CIP is frozen. A draft CIP can change.
- A rule states what a program does, never how.

## Write the code

The conventions are in [`AGENTS.md`](AGENTS.md). They apply to people and agents alike. The most important:

- Write a failing test first, then make it pass.
- A test proves behaviour through the public surface, and cites the spec section it proves:
  ``(`spec/trails/03-steps.md`, "Closing")``.
- A name is a noun for what a thing is. One primary type per file.
- Docstrings on types and members only. No inline comments.

## Verify

```bash
make verify        # fmt, clippy -D warnings, and the tests — what CI runs
make fmt           # write the formatting that verify checks
make install-cli   # build `celis` from this checkout into ~/.local/bin
```

To try the plugin from this checkout for one session, run `claude --plugin-dir plugins/celis` or
`cursor-agent --plugin-dir plugins/celis`.

## Open a pull request

- Branch from `main`, and keep one change in one pull request.
- Write the commit title as `<type>: <what changed>`, where the type is `feat`, `fix`, `docs`, `refactor`
  or `chore`.
- Say in the description which CIP the change follows, and how you checked it.

By contributing, you agree that your work is licensed under the [MIT License](LICENSE).
