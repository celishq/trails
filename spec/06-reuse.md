---
cip: 6
title: Reuse
status: draft
depends: [CIP-03, CIP-04, CIP-05]
---

# CIP-06 — Reuse

A step names the patterns it recalls and uses, and the earlier result it fixes. A fix of a step, or the
agent's mark, tells which patterns misled it.

## Recalls

```json
{ "kind": 1985, "content": "",
  "tags": [["L", "celis.step"], ["l", "recall", "celis.step"], ["e", "<step id>"],
           ["pattern", "<id of a live pattern>"]] }
```

- `pattern`: the pattern the agent read whole for the work of the step.

**Writers**

- A writer MUST write the whole id, and only the id of a pattern that is live in scope in the step's
  project (CIP-05, "Scope").
- A writer MUST write one recall at most for a step and a pattern.

**Readers**

- A recall is a read of the pattern for the step. It is not a use.
- A reader MUST ignore a recall that names no pattern it holds.

## Uses

```
["uses", "<id of a live pattern>"]     a step's tag, zero or more
```

**Writers**

- A writer MUST write the whole id, once, and only the id of a pattern that is live in scope in the
  step's project (CIP-05, "Scope") when it writes the step.

**Readers**

- A `uses` is a use of the pattern it names.
- A reader MUST ignore a `uses` that names a step, and one that names no record it holds.

## Revisions

A step's `replaces` (CIP-03, "Steps") MAY name a done step of any trail.

**Readers**

- The done step MUST stay done. It is revised by each step whose `replaces` names it, written at or after
  its first `done`.
- A step whose `uses` names a pattern misled the pattern once when the step is revised.

## Misled

```
["misled", "<id of a pattern>"]     a step's tag, zero or more: the step fixes work that the pattern misled
["misled", "<id of a pattern>"]     a done's tag, zero or more: the pattern misled the step's own work
```

**Writers**

- A writer MUST write the whole id, once, and only the id of a pattern that the store holds.
- A step MUST carry a `misled` only with a `replaces` of a done step.

**Readers**

- A pattern misled a step once when the step's `done` names it in `misled`, and when a step that
  revises the step names it in `misled`. Each misled step counts once for a pattern, with "Revisions".
- A reader MUST ignore a `misled` that names no pattern it holds.
