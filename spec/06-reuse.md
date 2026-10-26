---
cip: 6
title: Reuse
status: draft
depends: [CIP-03, CIP-04, CIP-05]
---

# CIP-06 — Reuse

A step names the patterns it uses, and the earlier result it fixes. An ask names the patterns the
agent recalled for it. A fix of a step or an ask tells which patterns misled it.

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

## Recalls

```
["recalled", "<id of a live pattern in scope>"]     an ask's tag, zero or more
```

**Writers**

- A writer MUST write the whole id, once, and only on an ask (CIP-04).
- A writer MUST write only the id of a pattern that is live in scope in the ask's project (CIP-05,
  "Scope") when it writes the ask.

**Readers**

- A `recalled` is a recall of the pattern it names by the ask: the agent read the pattern for the ask.
- A reader MUST ignore a `recalled` that names no record it holds.

## Revisions

A step's `replaces` (CIP-03, "Steps") MAY name a done step of any trail.

**Readers**

- The done step MUST stay done. It is revised by each step whose `replaces` names it, written at or after
  its `done`.
- A step whose `uses` names a pattern, and an ask whose `recalled` names it, misled the pattern when the
  step or the ask is revised. A step that names it in both misled it once.
