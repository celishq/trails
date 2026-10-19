<h1 align="center">Celis</h1>

<p align="center">A small CLI and a plugin for AI agents which allows them to learn from each other.</p>

<p align="center">
  <a href="https://github.com/celishq/trails/releases/latest"><img alt="Release" src="https://img.shields.io/github/v/release/celishq/trails"></a>
  <a href="https://github.com/celishq/trails/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/celishq/trails/actions/workflows/ci.yml/badge.svg?event=pull_request"></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue"></a>
</p>

<p align="center">
  https://github.com/user-attachments/assets/c3177c2c-f3b2-4fb8-8afa-4bf8626da524
</p>

## Why

- **Skills that build themselves.** You don't write skill files by hand. When your agent starts
  something new, it finds similar work done before and hands over what worked. It gets
  better with every job.
- **Your agent stops repeating mistakes.** Correct it once, in chat or in a code review, and
  it remembers the fix next time, even in another project.
- **Less hand-holding.** The agent checks its own work after every step, so it can go further
  on its own.
- **All your agents learn together.** Something Claude Code learned is there for Codex and
  Cursor too.
- **Run agents in parallel.** A swarm of agents can work on one job at once. When one of them
  learns a rule, the others get it at their next step, not after the session ends.
- **You can see what happened.** Every step comes with proof: a passing test, a command's
  output, a screenshot. You can also watch the steps show up while the agent works.

## How it works

Celis borrows an idea from the Jobs To Be Done methodology: say what you want done, not how to
do it. "Seniors pay 20% less" is a job. "Edit the cart" is just one way to do it, and maybe
not the right one.

In celis, every job gets split into smaller jobs. We call each of those a **step**, and a step
can split again. All the steps for one job together make its **trail**.

Trails use [nostr](https://nostr.org), an open protocol, as their data format: each record is a
note signed with your own key. celis never sends them to relays or anywhere else, so
everything stays on your machine.

Here's what happens when you give your agent a job:

1. **The session starts.** The plugin shows the agent how to work in steps.
2. **The agent looks back.** Before it begins, the agent goes through your old trails from
   all your projects and finds steps like the ones coming up. It brings back what worked, what
   didn't, and why.
3. **The agent works one step at a time.** It does a step, proves it worked, and only then
   picks the next one. It starts with the part it's least sure about and splits it up until
   each piece is clear. Steps are named by what they're for, not how they were done, so the
   same step turns up again in other projects.
4. **The agent learns from what went wrong.** When a step doesn't go as planned, the agent
   writes down what it learned as a **rule**, like "tests need SHELF_OFFLINE=1". When you
   correct the agent, or give it code review comments to fix, each point becomes a rule too.
   A rule can be for one project or for all of them.
5. **Subagents get the rules too.** When the agent hands a step to a subagent, the rules for
   that step go with it.

Here's a trail halfway through a job:

```
Job: seniors pay 20% less
├─ ✓ Find where prices are set
├─ ◐ Change how the price is worked out     ← not sure how yet, so it's split up
│  ├─ ✗ Change the price in the cart           dropped: the cart doesn't know the age
│  │                                           rule: set prices in prices.py, not the cart
│  ├─ ✓ Write a test: seniors pay 80%          proof: passes, once SHELF_OFFLINE=1 was set
│  │                                           rule: tests need SHELF_OFFLINE=1
│  └─ ○ Use the new price in prices.py      ← up next
└─ ○ Show the senior price on the page      ← will split this later

✓ done   ◐ in progress   ○ to do   ✗ dropped
```

A week later, you ask for a student discount. This is what the agent sees first:

```
New job: students pay 10% less

  ✓ seniors pay 20% less    40 min · no corrections   ← start from this trail

  rules:
  • tests need SHELF_OFFLINE=1
  • set prices in prices.py, not the cart
```

## Early results

> [!NOTE]
> These numbers come from our early evals, with each job run once. We're still gathering data, so
> take them as a first look, not a final benchmark.

We gave agents 8 real Supabase pull requests that build landing pages, all merged after the
model's training data. After two of them, the agent got the rules that Supabase's review bots had
asked for. Then we counted how often the later jobs broke those rules.

| Agent                                          | Review rules broken per job | Cost per job |
| ---------------------------------------------- | --------------------------- | ------------ |
| Claude Code                                    | 2.75                        | $0.31        |
| Claude Code with celis                         | 0                           | $0.31        |
| Codex                                          | 2.38                        | $0.04        |
| Codex with Celis, using rules from Claude Code | 0.25                        | $0.05        |

Celis didn't make the pages look more like the real ones. What it changed is that agents kept
the rules they had been given.

## Why not just CLAUDE.md?

A `CLAUDE.md` works well when the right rule is in it. In our evals, a `CLAUDE.md` that already held
the rule did as well as celis. The catch is that someone has to write it, and keep it current.

Celis writes the rules for you, from your corrections and code reviews, while the work happens.
At the start of each job, the agent gets the rules and past steps that matter for it, from every
project. And every agent gets them, whichever instruction file it reads.

## Install

**1. Install `celis`:**

```bash
curl -fsSL https://celis.ai/install.sh | sh
```

To update it later, run `celis update`.

**2. Add the plugin to your agent:**

| Agent       | Install                                                                         | Then                                                                                        |
| ----------- | ------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| Claude Code | `/plugin marketplace add celishq/trails`<br>`/plugin install celis@celis`       | —                                                                                           |
| Codex       | `codex plugin marketplace add celishq/trails`<br>`codex plugin add celis@celis` | Trust its hooks with `/hooks`. In a sandbox, let it write the folder celis keeps trails in. |
| Cursor      | `cursor-agent plugin marketplace add https://github.com/celishq/trails`         | Install `celis` from `/plugins`. Cursor doesn't remind the agent on each message.           |
| Gemini CLI  | `gemini extensions install https://github.com/celishq/trails`                   | —                                                                                           |
| Antigravity | `agy plugin install https://github.com/celishq/trails`                          | Antigravity has no prompt hook, so celis doesn't write your prompts on the trail.           |

**3. Give your agent a job,** and run `celis tree` in a second terminal to watch its steps.
Questions don't leave a trail; only jobs that change files do.

<details>
<summary>Keep celis data somewhere else</summary>

celis keeps your trails and your key in `~/Library/Application Support/Celis` on macOS, and in
`~/.local/share/celis` on Linux. To keep them in another folder, for a test or a
separate setup, set `CELIS_HOME` to that folder before you start the agent:

```bash
CELIS_HOME="$PWD/.celis-home" claude
```

</details>

## Uninstall

1. Remove the plugin from your agent the same way you added it. In Claude Code, run
   `/plugin uninstall celis@celis`.
2. Delete the binary:

   ```bash
   rm ~/.local/bin/celis
   ```

3. To delete your trails and your key too, delete the data folder. Trails live only on your
   machine, so this removes them for good.

   ```bash
   rm -rf ~/Library/Application\ Support/Celis   # macOS
   rm -rf ~/.local/share/celis                   # Linux
   ```

## Questions

**How do I fix a bad rule?**

Tell your agent. celis is agent native: your agent knows every command, so asking it is always
easier than running celis yourself. It writes a new rule that replaces the old one
(`celis decide --replaces <id>`). A rule can't be deleted, but once it's replaced, agents stop
seeing it.

**How do I share trails with my team?**

Run `celis link` in the project. It keeps an up-to-date copy of the project's trails in
`.celis/trails/` in the repo, so you can commit them and git syncs them like any other file.
You can link any other folder too, like a synced drive. Rules you keep for all your own work
stay private.

**Is a job one session?**

Not always. Your agent decides on its own how to split a job, and picks the way that gets it
done best. A job can take many sessions, and one session can work on many jobs. Subagents
share their parent's session.

**Where is my key, and how do I back it up?**

celis makes a 12-word recovery phrase the first time it saves a trail. Run `celis auth phrase`
to see it and write it down. On another computer, run `celis auth login` and type it in.

**What other commands are there?**

Run `celis help`. Besides `tree`, you can search all trails with `celis grep`, see what later
work reused with `celis stats`, and turn trails into Markdown for a pull request with
`celis report`.

**Does it work on Windows?**

Not yet natively. Run celis in WSL.

## Status

Celis is in early development. Releases are `0.0.x`, and the trail format is still a draft, so
it may change between versions.

What works today:

- Claude Code, Codex, Cursor, Gemini CLI and Antigravity
- macOS and Linux, and Windows through WSL
- Sharing trails with your team through git, with `celis link`

## Talk to us

Celis is new, and we'd love to hear from you. If you want help getting started, or you have
feedback of any kind, [tell us here](https://tally.so/r/pbl09Z).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
