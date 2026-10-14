---
cip: 6
title: Reuse
status: draft
depends: [CIP-01, CIP-03, CIP-04]
---

# CIP-06 — Reuse

A step names the past work it builds on and the earlier result it fixes. A reader can then tell which
paths later work followed, which rules it kept, what a follow saved, and which results did not hold.

## Uses

```
["uses", "<id of a live decision, or of a step of another trail>"]     a step's tag, zero or more
```

**Rules**

- A writer MUST write the whole id, once. An id that names a trail MUST be written as the id of the step
  that holds its work: its first ask when that ask has the trail's text (CIP-04, "Sessions"), else the
  trail.
- A writer MUST NOT write a `uses` that names a step of the step's own trail.
- When a writer writes a trail and its first ask together, the `uses` tags MUST go on the ask only.
- A reader MUST read a `uses` that names a decision as a use of that decision.
- A reader MUST read a `uses` that names a step as a follow: the step follows that step, its source.
- A reader MUST ignore a `uses` that names no record it holds.

## Revisions

A step MAY carry a `replaces` (CIP-03, "Steps") that names a done step, of any trail.

**Rules**

- The done step MUST stay done.
- The writer of a trail and its first ask together MUST put the `replaces` on the ask only.
- A reader MUST read a done step as revised by each step whose `replaces` names it, written at or after
  its `done`.
- A reader MUST read a trail as `revised` when it would be done (CIP-04, "State"), and a done step of
  its tree is revised.

## Derived

```
followers   the steps that follow a step, and the steps that follow those, at any depth, each once
sources     the steps a step follows, and the steps those follow, at any depth, each once
used        the number of steps whose uses name a decision
saved       took of the source less took of the follower (CIP-03, "Grades"); below zero when it took more
```

**Rules**

- A reader MUST work out each of these from the records alone. They MUST NOT be records.
- A saving MUST count only when both steps are done.
- The saving of a step MUST be the sum of the savings of each follow at any depth below it, each against
  the source it names.

## Periods

A period is the span of time from one second up to another. Stats compares two of them: the last 14 days,
and the 14 days before.

```
done        the steps whose done record falls in it
planned     the done steps with no `changed` (CIP-03, "Closing")
reworked    the done steps that a step revised (Revisions)
added       the steps written in it
ruled       the added steps whose uses name a decision
closed      the steps done or dropped in it, and the expired steps of a trail whose newest record falls in it
follows     the follows whose step is done in it, and the sum of their savings
```

**Rules**

- A reader MUST NOT count a trail as a step of a period.
- A share MUST be the count of its part in its base: planned and reworked in done, ruled in added.
- A reader MUST NOT show a change of a share or of a saving when either period counts fewer than 10
  records in its base.
