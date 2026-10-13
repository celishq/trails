# celis trails

Trails for coding agents. Every record is a [nostr](https://nostr.com) note that your account signs.

## How it works

A big task fails when an agent holds all of it at once. celis splits it into a trail:

- **A trail is one task, in your words.** The agent breaks it into steps by their logical parts, from the
  big shapes of the work down to one small action at a time.
- **A step closes with proof.** The agent marks it done with a result that a person can check, or drops it
  with a reason.
- **A decision keeps what the agent learned.** "The tests need `SHELF_OFFLINE=1`" goes on the file it is
  about, so the next agent at that file reads it first.
- **Later work starts from past work.** At each new task, the agent gets the trails like it from every
  project on your machine. It uses again the steps and rules that worked, and does not learn them twice.

You see the trail as it grows with `celis tree`. The contract is in [`spec/trails`](spec/trails).

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

Read [CONTRIBUTING.md](CONTRIBUTING.md) before you open a pull request.

## License

MIT
