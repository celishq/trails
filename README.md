# celis trails

Trails for coding agents. A trail is one task broken into steps that agents take and close, with the
decisions made on the way, each at its place. Later sessions load them, so an agent keeps the rules the
last one learned. Every record is a nostr note that your account signs.

- `crates/celis` — the `celis` command line.
- `crates/celis-trails` — the trails and decisions, and the store they live in.
- `crates/celis-nostr` — the nostr wire: the account from a recovery phrase, signed notes, NIP-44 and
  NIP-59 encryption, and the relay client.
- `plugins/celis` — the plugin that gives an agent the trail at the start of each session, for Claude Code,
  Codex, Cursor and Gemini CLI.
- `spec/trails` — the contract of the trail, as numbered CIPs.

## Install

```bash
curl -fsSL https://celis.ai/install.sh | sh
```

Or build it from this checkout:

```bash
make install-cli   # put `celis` in ~/.local/bin
```

## Use it in Claude Code

The plugin needs `celis` on your PATH. Load it for one session:

```bash
claude --plugin-dir ~/path/to/celis-trails/plugins/celis
```

Or install it for every session:

```
/plugin marketplace add celishq/trails
/plugin install celis@celis
```

## Use it in other agents

Each agent reads its own manifest in this repo, and runs the same hooks. Each needs `celis` on your PATH.

- Codex: `codex plugin marketplace add celishq/trails`, then `codex plugin add celis@celis`. Trust its
  hooks with `/hooks`.
- Cursor: `cursor-agent plugin marketplace add https://github.com/celishq/trails`, then install `celis`
  from `/plugins`. Or load it for one session with `cursor-agent --plugin-dir
  ~/path/to/celis-trails/plugins/celis`. Cursor gives the agent no note on each prompt.
- Gemini CLI: `gemini extensions install https://github.com/celishq/trails`.

Give the agent a task. It keeps a trail when the task changes files. Run `celis tree --watch` in a second
terminal to see the trail while the agent works.

## Develop

```bash
make verify   # fmt, clippy and the tests — what CI runs
make fmt      # write the formatting that verify checks
```

## License

MIT
