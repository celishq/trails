---
cip: 6
title: Reuse
status: draft
depends: [CIP-03, CIP-04, CIP-05]
---

# CIP-06 — Reuse

A step names the patterns it uses, and the earlier result it fixes. A fix of a step tells which patterns
misled it.

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
  its `done`.
- A step whose `uses` names a pattern misled the pattern once when the step is revised.
