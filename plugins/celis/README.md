# celis

No more "You're absolutely right!"

Stop telling your agent the same thing twice. Celis turns your corrections into rules that Claude Code, Codex,
Cursor and Gemini remember in every project. Your trails and rules stay on your machine.

## Before you install

The plugin needs the `celis` command. Install it first:

```bash
curl -fsSL https://celis.ai/install.sh | sh
```

To read the script before it runs, or to check the download, see
[Install](https://github.com/celishq/trails#install). Then add the plugin in Claude Code:

```
/plugin install celis --marketplace celishq/trails
```

## What it runs

- At the start of a session, a hook runs `celis` and gives the agent the steps to follow. When `celis` is
  missing, it tells the agent how you install it.
- On each prompt, a hook runs `celis`, which writes the prompt on the session's open trail.
- The skills and the `past` agent run `celis` commands that read and write your trails.

The plugin downloads nothing and sends nothing. [SECURITY.md](https://github.com/celishq/trails/blob/main/SECURITY.md)
says what `celis` keeps, sends and runs.
