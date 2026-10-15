---
cip: 4
title: Trails
status: draft
depends: [CIP-01, CIP-02, CIP-03]
---

# CIP-04 — Trails

A trail is one task you give, in your words, and the steps agents take toward it. This CIP lets a reader
know which trail a step belongs to, which session works on it, what the person asked on the way, which
rules hold in it, whether it was reached, dropped or left, which of its work was revised, and which past work is like a new ask.

## Trails

A trail is a step with no `under` (CIP-03, "Steps"). The steps under it, and the steps under those, are
its tree. An ask is a step right under a trail.

**Meaning**

- A reader MUST count a step with no `under` as a trail.
- A reader MUST count a step in the tree of the trail at the top of its `under` chain.

**Examples**

```
"The price list shows a senior tier at 80% of the price, and the receipt shows it."
"Make the badges orange."
```

## Sessions

A session is one run of an agent, and a subagent shares it. Claude Code names it in
`CLAUDE_CODE_SESSION_ID`, and Codex in `CODEX_THREAD_ID`. Other agents give it to their hooks only, so the
hook names it to the agent, and the agent passes it with `--session`. Many sessions can work on one trail.
The prompt hook writes the person's prompts on the session's trail.

```
["session", "<session id>"]                  a step's tag, optional: the session that wrote it
```

```json
prompt  kind 1985, signed by the account (NIP-32)
{ "kind": 1985, "content": "<the prompt's first line, 200 characters at most>",
  "tags": [["L", "celis.step"], ["l", "prompt", "celis.step"], ["e", "<trail id>"]] }
```

**Meaning**

- A writer MUST write a `session` tag on each step, when it has a session: `--session`, else the first
  set variable of `CLAUDE_CODE_SESSION_ID`, `CODEX_THREAD_ID` and `CELIS_SESSION_ID`.
- A trail with a session MUST hold each ask as a step under it, the first ask too. So a writer MUST write
  the trail, then the ask under it with the same text.
- A step with no `under`, while its session has an open trail, MUST go under that trail, unless it asks
  for a new trail (`--new`). Else it MUST be a new trail. It MUST NOT join another session's trail
  without `--under`.
- The prompt hook MUST write a prompt as a `prompt` on the session's open trail, and only there. With no
  open trail, it MUST write nothing.
- Its text MUST be the prompt's first line, cut to 200 characters.
- The hook MUST write nothing for a prompt that starts with `<task-notification>` or `<agent-message`.
  The harness wrote it, not the person.
- A reader MUST ignore a kind 1985 record with `steer` in `celis.step`. Older versions wrote it.

**Derived**

- A session's open trail is the trail of its newest step, while that trail is not done or dropped.
- A prompt counts in an ask's review when it was written while that ask was open (CIP-03, "Reviewing").

**Examples**

```
CLAUDE_CODE_SESSION_ID=95e9be99 celis step add -- "Make the badges orange."    a trail, the ask under it
CLAUDE_CODE_SESSION_ID=95e9be99 celis step add -- "Now the buttons too."       a step under it
CLAUDE_CODE_SESSION_ID=95e9be99 celis step add --new -- "Fix the login page."  a second trail
CLAUDE_CODE_SESSION_ID=4c1d22e0 celis step add -- "Check the colours."        a trail of its own
CLAUDE_CODE_SESSION_ID=4c1d22e0 celis step add --under 3fa2c1d0 -- "Check the colours."  goes there
celis step add --session 95e9be99 -- "Now the links."                        a step under its trail
```

## Rules

A trail's rules are the live decisions with the scope `trail:<id>` (CIP-01, "Scope"). Each is one
sentence a check can fail.

**Meaning**

- A reader MUST count a decision as a rule of a trail only when its scope names that trail.

**Examples**

```
celis decide --scope trail:4c1d22e0 -- "A senior pays 80% of the price, rounded down to the cent."
```

## State

A trail has no state record. It takes the states of a step (CIP-03, "Starting"), worked out from its
tree, so the person can see which tasks the agents
reached, and which work a later step revised. A trail ends when no step under it is open, or 30 minutes
after the newest record of its tree.

**Derived**

```
open      not done, dropped or ended, and no step of its tree works
working   not done, dropped or ended, and a step of its tree works
done      done, or ended with no open step under it
dropped   the trail is dropped
expired   ended with an open step under it
```

- The newest record of a tree is the newest of its steps and the records that name them.
- A step of an ended trail that is not done or dropped is `expired`. A closed step keeps its state.

**Meaning**

- A reader MUST work out a trail's state from its tree, their `start`, `done` and `drop` records, the
  steps that replace them, and the time.
- A trail with steps under it MUST end when none of them is open.
- A trail MUST end 30 minutes after the newest record of its tree.
- A new step under an ended trail MUST make it open again.
- A trail is active while it is open or working.
- A reader MUST NOT count a step of an ended trail as open.

## The past

The past shows how a task like a new one was done, so a new plan can follow the path that worked. An
agent chooses it: `past` lists, and the plugin's `celis:past` agent reads the list and opens the trees.
The list starts with what a new ask of the project must keep: its active trails and its decisions.

**Derived**

- What a new ask must keep: the project's active trails, each with its rules, and the live decisions in
  scope (CIP-01, "Reading").
- Past work: a done ask, or a done trail, of every project.
- A trail with an ask of its own text is no past work, so the ask stands for it.
- The rules of a past work: the rules of the trail it is, or of the trail it stands for. They stay after
  the trail ends, so the next ask can keep what the trail learned.

> Why an agent chooses: it reads an ask in any language, and it tells the same kind of task from the same
> words. The list stays one line a work, so it fits the agent's context for thousands of asks.
