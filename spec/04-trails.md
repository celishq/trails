---
cip: 4
title: Trails
status: draft
depends: [CIP-01, CIP-03, CIP-05]
---

# CIP-04 — Trails

A trail is one task the person gives, in their words, and the steps agents take toward it.

## Trails

A trail is a step with no `under` (CIP-03). The steps under it, at any depth, are its tree. An ask is a
step right under a trail.

**Readers**

- A reader MUST count a step in the tree of the trail at the top of its `under` chain.

## Sessions

A session is one run of an agent; a subagent shares it. Many sessions can work on one trail.

```
["session", "<session id>"]     a step's tag
```

```json
{ "kind": 1985, "content": "<the prompt's first line, 200 characters at most>",
  "tags": [["L", "celis.step"], ["l", "prompt", "celis.step"], ["e", "<trail id>"]] }
```

- `session`: `--session`, else the first set variable of `CLAUDE_CODE_SESSION_ID`, `CODEX_THREAD_ID` and
  `CELIS_SESSION_ID`.
- `prompt`: what the person wrote while the trail was open.

**Writers**

- A writer MUST write a `session` tag on each step that has a session.
- A session's open trail is the trail of its newest step, while it is not done or dropped.
- A trail with a session MUST hold each ask under it, the first too: the writer writes the trail, then the
  ask with the same text.
- A step with no `under` MUST go under its session's open trail, unless it asks for a new trail, else be
  a new trail. It MUST NOT join another session's trail without `under`.
- The prompt hook MUST write a prompt on its session's open trail only. With no open trail, the
  session's next `step add` MUST write the prompt on the trail of its step. The hook MUST write nothing
  for a prompt that starts with `<task-notification>` or `<agent-message`.

## Notes

A trail's notes are the notes on the steps of its tree (CIP-03, "Notes"), and each pattern that CIP-05,
"Reading", reads as a note on the trail.

## State

**Readers**

- A trail takes the states of a step (CIP-03, "Starting") from its own records.
