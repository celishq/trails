---
cip: 6
title: Reuse
status: draft
depends: [CIP-03, CIP-04, CIP-05]
---

# CIP-06 — Reuse

A step names the past work it builds on, and the earlier result it fixes.

## Uses

```
["uses", "<id of a live decision, or of a step of another trail>"]     a step's tag, zero or more
```

**Writers**

- A writer MUST write the whole id, once. An id that names a trail MUST be written as its first ask when
  that ask has the trail's text (CIP-04, "Sessions"), else as the trail.
- A writer MUST NOT write a `uses` that names a step of the step's own trail.
- A writer of a trail and its first ask together MUST put `uses` and `replaces` on the ask only.

**Readers**

- A `uses` that names a decision is a use of it. One that names a step is a follow of that step.
- A reader MUST ignore a `uses` that names no record it holds.

## Revisions

A step's `replaces` (CIP-03, "Steps") MAY name a done step of any trail.

**Readers**

- The done step MUST stay done. It is revised by each step whose `replaces` names it, written at or after
  its `done`.
