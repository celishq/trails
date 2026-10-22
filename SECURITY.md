# Security

## Report a vulnerability

Report it in private from the [Security tab](https://github.com/celishq/trails/security/advisories/new) of
this repo. Do not open a public issue for it.

## What celis keeps

celis keeps your trails, rules, prompts and account phrase on your machine only: in
`~/Library/Application Support/Celis` on macOS and `~/.local/share/celis` on Linux, or in `$CELIS_HOME`. The
phrase file can be read by its owner alone. `celis link` copies a project's trails into a folder of that
project, `.celis/trails` by default, so they go wherever you commit or push that folder.

## What celis sends

The `celis` command connects to the network for `celis update` only. It downloads `install.sh` of the latest
[release](https://github.com/celishq/trails/releases) and runs it, and the script downloads the archive and
checks its checksum. celis sends no telemetry.

The `issue` skill writes a report of a celis fault. It shows you the report, and sends it as a GitHub issue
only when you say yes.

## What the plugin runs

The plugin's hooks run short shell scripts in `plugins/celis/hooks/`. Each one only runs the `celis`
command, or says how to install it when it is missing:

- `celis hook start`, at the start of a session, prints how to use celis.
- `celis hook ask`, on each prompt, writes the prompt on the session's open trail and prints a note for the
  agent.

Claude Code runs `start.sh` and `ask.sh`. The other agents run `run.sh`.

The plugin downloads nothing and installs nothing. You install `celis` yourself.

## Check a release

Each archive has a checksum and a build attestation from this repo's release workflow. To check one with the
[GitHub CLI](https://cli.github.com):

```bash
gh attestation verify celis-aarch64-apple-darwin.tar.gz -R celishq/trails
```
