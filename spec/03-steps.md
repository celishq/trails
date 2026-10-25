---
cip: 3
title: Steps
status: draft
depends: [CIP-01]
---

# CIP-03 — Steps

A step is one part of the work, at a place. An agent leaves it for itself or the next agent, takes it up,
and closes it with a proof, or drops it.

## Steps

```json
{ "kind": 1111,
  "tags": [
    ["I", "project:<project id>"], ["K", "project"],
    ["i", "project:<project id>"], ["k", "project"],
    ["project", "<project name>"],
    ["under", "<id of the step it is a part of>"],
    ["replaces", "<id of the step whose work it takes over>"],
    ["at", "<a place the work goes>"],
    ["check", "<the command that proves the work>"],
    ["session", "<the session that wrote it>"],
    ["L", "celis.step"], ["l", "step", "celis.step"]
  ],
  "content": "<the title>\n<the detail>" }
```

- `under`: optional; the step this one is a part of. A step with none is a trail (CIP-04).
- `replaces`: optional; an open step whose plan it takes over, or a done step it revises (CIP-06).
- `at`: zero or more; a place of the work (CIP-01, "Places").
- `check`: optional; the command that proves the work.
- `session`: optional; CIP-04, "Sessions".
- content: the work, so a stranger with only the step can do it. Its first line is the title.

**Writers**

- A step MAY carry one `under`, one `replaces`, one `check`, and any number of `at`.

**Readers**

- A reader MUST count a kind 1111 record as a step only with the label `step` in `celis.step`, and never
  as a pattern.

## Starting

```json
{ "kind": 1985, "content": "",
  "tags": [["L", "celis.step"], ["l", "start", "celis.step"], ["e", "<step id>"]] }
```

**Readers**

```
open      no `start`, `done` or `drop` names the step
working   a `start` names it, and no `done` or `drop`
done      a `done` names it
dropped   a `drop` names it
```

- A `start` MUST NOT close its step.

## Closing

```json
{ "kind": 1985, "content": "<the proof>",
  "tags": [["L", "celis.step"], ["l", "done", "celis.step"], ["e", "<step id>"],
           ["changed", "<how the work differs from the plan>"],
           ["by", "judge"]] }
```

- content: the proof, the check that failed before and passes now.
- `changed`: optional; with none, the work went as planned.
- `by`: optional; who gave the proof: `judge`, a reader that did not do the work, or `person`. With none,
  the proof is the agent's own.

**Writers**

- A writer MUST NOT write a `done` for a step with an open step under it, or whose check fails.

**Readers**

- The first `done` or `drop` that names a step MUST settle it.
- A reader MUST ignore a `done` with no content.

## Dropping

```json
{ "kind": 1985, "content": "<why the work is no longer wanted>",
  "tags": [["L", "celis.step"], ["l", "drop", "celis.step"], ["e", "<step id>"]] }
```

**Writers**

- A writer that drops a step MUST also drop each open step under it, at any depth.
- A step that `replaces` an open step MUST go under that step's `under` when it names none. Its writer
  MUST then drop the old step and its open steps with `replaced by <new id, first 8>`.
- A step MUST NOT replace a trail or a dropped step.
- A writer MAY drop a done trail, and MUST NOT drop another done step.

**Readers**

- A reader MUST ignore a drop with no content.
- A drop of a done trail MUST keep it done. The trail is then **dropped after done**. Its patterns stay
  live.

## Editing

```json
{ "kind": 1985, "content": "<a new title, or a title and the detail; may be empty>",
  "tags": [["L", "celis.step"], ["l", "edit", "celis.step"], ["e", "<step id>"],
           ["at", "<a place>"],
           ["check", "<the command that proves the work>"]] }
```

**Readers**

- A reader MUST apply a step's edits oldest first.
- One line of content MUST change only the title. More lines MUST replace the text. No content MUST keep
  it.
- `at` tags MUST replace the step's places, and a `check` its check.

## Sources

**Readers**

- A reader MUST read steps from the store only (CIP-01). A project that keeps `@todo` lines in its code
  adds each as a step.
