---
cip: 3
title: Steps
status: draft
depends: [CIP-01, CIP-02]
---

# CIP-03 — Steps

A step is one part of the work, at a place. An agent leaves it for itself or the next agent, takes it up,
and closes it with a proof or drops it. This CIP lets a reader know what work is open, what is done and
with what proof, how the work differs from its plan, and how long each step took.

## Steps

A step is a NIP-22 comment on its project, like a decision (CIP-01, "Decisions"). Its text states the
work, so a stranger with only the step can do it. The first line of its text is its title, and the lines
after it hold the detail. A step with no `under` is a trail (CIP-04).

```json
{
  "kind": 1111,
  "tags": [
    ["I", "project:<project id>"], ["K", "project"],
    ["i", "project:<project id>"], ["k", "project"],
    ["project", "<project name>"],
    ["under", "<id of the step it is a part of>"],                    optional
    ["replaces", "<id of the step whose work it takes over>"],        optional
    ["at", "<a place the work goes: file:line, a URL, any text>"],    zero or more
    ["check", "<the command that proves the work>"],                  optional
    ["session", "<the session that wrote it>"],                       optional (CIP-04)
    ["L", "celis.step"],
    ["l", "step", "celis.step"]
  ],
  "content": "<the work>"
}
```

```
under      the step this one is a part of
replaces   an open step whose plan it takes over, or a done step of any trail whose work it revises
at         a place the work goes (CIP-01, "Places")
check      the command that proves the work
```

**Meaning**

- A step MUST carry content, the anchor `project:<project id>` and the project name.
- A step MAY carry one `under`, one `replaces`, one `check`, and any number of `at`.
- A reader MUST count each `file` tag of a step as a place, after its `at` places.
- A reader MUST count a kind 1111 record as a step only when it carries the label `step` in
  `celis.step`. It MUST ignore tags it does not know.
- A reader of decisions MUST NOT count a step as a decision (CIP-01, "Decisions").

**Derived**

- A step's title is the first line of its text. The other lines are its detail.

**Examples**

```
"Write a test: undo restores the cursor (spec/editing.md, \"Undo\")."     at crates/editor/src/undo.rs:40
"Make the onboarding title use the new type scale."                       at https://figma.com/design/Ab12/…
"Round the senior price down\nThe test expects 7.99 for 9.99, not 8.00."   a title and its detail
```

## Projects

A step names a project as CIP-01 "Projects" states. Outside git, the place of the work names it.

```
project id   inside git: as CIP-01 "Projects" states
             outside git, with a first place that is a URL:
               figma:<file key>   a figma.com URL, /file/<key>/… or /design/<key>/…
               web:<host>         any other URL
             else: as CIP-01 "Projects" states
project name the file name in a Figma URL, else the host, else as CIP-01 "Projects" states
```

**Meaning**

- A step written outside git, with a first place that is a URL, MUST name the project of that URL.
- Two steps at the same Figma file MUST name the same project id, whatever the working folder.
- Each step MUST carry the project name its own URL holds. Two steps of one project MAY carry two
  names.

> **Why the place.** A design team never runs `celis` in a repo. Its work sits in a Figma file, and the
> file key stays the same for every person who opens it.

## Starting

A step starts when an agent takes it up.

```json
start  kind 1985, signed by the account (NIP-32)
{ "kind": 1985, "content": "",
  "tags": [["L", "celis.step"], ["l", "start", "celis.step"], ["e", "<step id>"]] }
```

**Meaning**

- A `start` MUST NOT close its step.

**Derived**

```
open      no `start`, `done` or `drop` names the step
working   a `start` names the step, and no `done` or `drop` does
done      a `done` names the step
revised   a `done` names the step, and a later step replaces it (CIP-06, "Revisions")
dropped   a `drop` names the step
expired   not done or dropped, and its trail ended (CIP-04, "State")
```

- A reader works out a step's state from its `start`, `done` and `drop` records.

## Closing

A step closes when it is done. A done step carries its proof, and what changed from its plan.

```json
done   kind 1985, signed by the account (NIP-32)
{ "kind": 1985, "content": "<the proof: the check that failed before and passes now>",
  "tags": [["L", "celis.step"], ["l", "done", "celis.step"], ["e", "<step id>"],
           ["changed", "<how the work differs from the step's title>"]] }      changed: optional
```

```
content   the proof
changed   how the work differs from the plan; with none, it went as planned
```

**Meaning**

- A step MUST be open until a `done` or a `drop` names it. The first of them MUST settle it.
- A reader MUST ignore a `done` record with no content. It does not close the step.
- A writer MUST NOT write a `done` for a step with an open step under it.
- A writer MUST NOT write a `done` for a step whose check fails.

## Reviewing

A review compares a done step with its plan, so the agent adds the next step only after it has seen the
last one land. A step states its plan in its title, and in a line of its text that starts with `expect:`.
A review is no record.

**Derived**

```
expect    the text after `expect:`, when the step has the line
planned   the steps under it written before the first of them started
added     each step under it written after the first of them started
dropped   each step under it that is dropped, with its reason
changed   each done step under it with a `changed`, then its own
asked     on an ask, each prompt on its trail written while the ask was open (CIP-04, "Sessions")
open      each open step under the step's `under`, other than the step
```

## Dropping

A step is dropped when its work is no longer wanted. A drop closes it without a proof.

```json
drop   kind 1985, signed by the account (NIP-32)
{ "kind": 1985, "content": "<why the work is no longer wanted>",
  "tags": [["L", "celis.step"], ["l", "drop", "celis.step"], ["e", "<step id>"]] }
```

**Meaning**

- A reader MUST ignore a drop with no content. It does not close the step.
- A writer that drops a step MUST also drop each open step under it, at any depth.
- A step that `replaces` an open step MUST go under that step's `under` when it names no `under`. Its
  writer MUST then drop the old step and each open step under it, with `replaced by <new id, first 8>`.
- A step MUST NOT replace a trail or a dropped step.
- A done step that a later step replaces MUST stay done.

**Derived**

- A dropped step has no grade.

## Editing

An edit gives a step a new title or text, new places, or a new check. A trail takes more asks as its
session goes on, so its first title stops covering the work; the agent then edits it.

```json
edit   kind 1985, signed by the account (NIP-32)
{ "kind": 1985, "content": "<a new title, or a title and the detail; may be empty>",
  "tags": [["L", "celis.step"], ["l", "edit", "celis.step"], ["e", "<step id>"],
           ["at", "<a place>"],                                          zero or more
           ["check", "<the command that proves the work>"]] }           optional
```

**Meaning**

- A reader MUST apply a step's edits oldest first.
- An edit with one line of content MUST change only the title, and keep the detail.
- An edit with more lines MUST replace the text. One with no content MUST keep the text.
- An edit with `at` tags MUST replace the step's places. One with a `check` MUST replace its check.

## Sources

The store is the only source of steps. A project that keeps `@todo` lines in its code adds each one as a
step, at the line's place.

**Meaning**

- A reader MUST read steps from the store only. It MUST ignore a `done` that names its step with a
  `step` tag in place of an `e` tag.

> **Why the store only.** A project chooses whether its code holds step lines. The core then knows one
> kind of step, and works the same inside git and outside it.

## Grades

A grade shows how the work of a done step went, so a new plan can follow the best earlier path. A grade is
no record.

**Derived**

```
took      the time from the step's first `start` to the `done` that settled it; with no `start`, from
          the step
revised   the steps that replace it after it was done
```

**Meaning**

- A reader MUST work out a grade from the records alone.
