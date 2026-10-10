---
cip: 12
title: Steps
status: draft
depends: [CIP-10, CIP-11]
---

# CIP-12 — Steps

## Steps

A step is one part of the work. An agent leaves it for itself or the next agent. It is a NIP-22 comment
on its project, like a decision (CIP-10, "Decisions"). Its text states the work, so a stranger with only
the step can do it. A step with no `under` is a trail (CIP-14).

```json
{
  "kind": 1111,
  "tags": [
    ["I", "project:<project id>"], ["K", "project"],
    ["i", "project:<project id>"], ["k", "project"],
    ["project", "<project name>"],
    ["under", "<id of the step it is a part of>"],                    optional
    ["fixes", "<id of a done step whose work it fixes>"],             optional
    ["follows", "<id of the past trail whose path it follows>"],      optional, a trail only (CIP-14)
    ["at", "<the place the work goes: file:line, a URL, any text>"],  optional
    ["file", "<a file the work may change or create>"],               zero or more
    ["check", "<the command that proves the work>"],                  optional
    ["L", "celis.step"],
    ["l", "step", "celis.step"]
  ],
  "content": "<the work>"
}
```

**Rules**

- A step MUST carry content, the anchor `project:<project id>` and the project name.
- A step MAY carry one `under`, one `fixes`, one `follows`, one `at`, one `check`, and any number of
  `file`.
- A reader MUST count a kind 1111 record as a step only when it carries the label `step` in
  `celis.step`. It MUST ignore tags it does not know.
- A reader of decisions MUST NOT count a step as a decision (CIP-10, "Decisions").

**Examples**

```
"Write a test: undo restores the cursor (spec/editing.md, \"Undo\")."     at crates/editor/src/undo.rs:40
"Make the onboarding title use the new type scale."                       at https://figma.com/design/Ab12/…
```

## Projects

A step names a project as CIP-10 "Projects" states. Outside git, the place of the work names it.

```
project id   inside git: as CIP-10 "Projects" states
             outside git, with an `at` that is a URL:
               figma:<file key>   a figma.com URL, /file/<key>/… or /design/<key>/…
               web:<host>         any other URL
             else: as CIP-10 "Projects" states
project name the file name in a Figma URL, else the host, else as CIP-10 "Projects" states
```

**Rules**

- `step add` outside git, with an `at` that is a URL, MUST name the project of that URL.
- Two steps at the same Figma file MUST name the same project id, whatever the working folder.
- Each step MUST carry the project name its own URL holds. Two steps of one project MAY carry two
  names.

> **Why the place.** A design team never runs `celis` in a repo. Its work sits in a Figma file, and the
> file key stays the same for every person who opens it.

## Starting

A step starts when an agent takes it up. A started step is working until it closes, so the person sees
which step the agent is on.

```json
start  kind 1985, signed by the account (NIP-32)
{ "kind": 1985, "content": "",
  "tags": [["L", "celis.step"], ["l", "start", "celis.step"], ["e", "<step id>"]] }
```

```
open      no `start` and no `done` names the step
working   a `start` names the step, and no `done` does
done      a `done` names the step
```

**Rules**

- A reader MUST work out a step's state from its `start` and `done` records.
- A `start` MUST NOT close its step.

## Closing

A step closes when it is done. A done step carries its proof.

```json
done   kind 1985, signed by an agent (NIP-32)
{ "kind": 1985, "content": "<the proof: the check that failed before and passes now>",
  "tags": [["L", "celis.step"], ["l", "done", "celis.step"], ["e", "<step id>"]] }
```

**Rules**

- A step MUST be open until a `done` names it. The first `done` MUST settle it.
- A reader MUST ignore a `done` record with no content. It does not close the step.

## Steering

A steer records what the person asked for while a step was open. The agent writes it, in one sentence, so
the trail shows which steps the person had to steer.

```json
steer  kind 1985, signed by an agent (NIP-32)
{ "kind": 1985, "content": "<what the person asked, in one sentence>",
  "tags": [["L", "celis.step"], ["l", "steer", "celis.step"], ["e", "<step id>"]] }
```

**Rules**

- A steer MUST NOT close its step.
- A reader MUST ignore a steer with no content.

## Sources

The store is the only source of steps. A project that keeps `@todo` lines in its code adds each one with
`step add`, at the line's place.

**Rules**

- A reader MUST read steps from the store only. It MUST ignore a `done` that names its step with a
  `step` tag in place of an `e` tag.

> **Why the store only.** A project chooses whether its code holds step lines. The core then knows one
> kind of step, and works the same inside git and outside it.

## The commands

```
celis step add [--under <id>] [--follows <id>] [--fixes <id>] [--at <place>] [--file <path>]... [--check <command>] -- <text>
                                                               leave a step
celis step start <id>                                         take up a step
celis step done <id> -- <proof>                               close a step with its proof
celis step steer <id> -- <text>                                record what the person asked
```

**Rules**

- Every command MUST take an id as CIP-11 "Output and errors" states.
- A command MUST refuse 8 characters that more than one step's id starts with, with `no-step`.
- `add` MUST refuse empty text as a usage error.
- `add` MUST refuse an `--under` or a `--fixes` that no step has, with `no-step`. It MUST refuse an
  `--under` step that is closed, with `closed`.
- `add` MUST carry the whole ids of `--under` and `--fixes`, each `--file` and the `--check` as their tags.
- `add` MUST NOT write a step when an open step of the same project has the same text. It MUST print
  the id of that step.
- `start` MUST refuse an id no step has, with `no-step`, and a closed step, with `closed`. On a working
  step, it MUST write nothing and print the id of the step's first `start`.
- `done` MUST refuse an id no step has, with `no-step`, and a closed step, with `closed`.
- `done` MUST refuse an empty proof, as a usage error.
- `done` MUST refuse a step with an open step under it, with `open-steps`, and write nothing.
- `done` MUST run the step's `check` in the working folder, when it has one. When the check fails, it
  MUST refuse with `check-failed`, print the end of the check's output, and write nothing.
- `steer` MUST refuse an id no step has, with `no-step`, and a closed step, with `closed`. It MUST refuse
  empty text as a usage error.
- `add`, `start`, `done` and `steer` MUST print the id of the record they wrote. With `--json`, each prints
  `{"id"}`, one a line.

```
text   <id, first 8> [<project name>] <text>                a step
       <id, first 8> [<project name>] <text> · <at>          a step with an `at`
json   {"id", "created_at", "project", "under", "fixes", "follows", "at", "files": [], "check", "content"}
```

**Examples**

```
celis step add --under 3fa2c1d0 --at shelf/prices.py:40 \
  -- "Write a test: a senior pays 80% of the price."
celis step add --under 3fa2c1d0 --fixes 4c1d22e0 -- "Round the senior price down, not up."
celis step add --under 3fa2c1d0 --at https://figma.com/design/Ab12/Shop \
  -- "The price list frame shows the senior tier."
celis step start 4c1d22e0
celis step done 4c1d22e0 -- "python3 -m unittest: test_senior failed before, passes now."
```

## Grades

A grade shows how the work of a closed step went, so a new plan can follow the best earlier path.

```
took      the time from the step to the `done` that settled it
rework    the steps whose `fixes` names it
steered   the steers that name it
```

**Rules**

- A reader MUST work out a grade from the records alone. A grade MUST NOT be a record.
- `tree` MUST print a done step's grade under it, after its proof.

```
text   took <45s | 1m 30s | 2h 5m | 3d 2h> · rework <n> · steered <n>
json   "grade": {"took": <seconds>, "rework": 0, "steered": 0}
```
