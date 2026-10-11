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
the step can do it. A step with no `under` is a trail (CIP-14). The first line of its text is its title,
and the lines after it hold the detail.

```json
{
  "kind": 1111,
  "tags": [
    ["I", "project:<project id>"], ["K", "project"],
    ["i", "project:<project id>"], ["k", "project"],
    ["project", "<project name>"],
    ["under", "<id of the step it is a part of>"],                    optional
    ["fixes", "<id of a done step whose work it fixes>"],             optional
    ["replaces", "<id of an open step whose plan it takes over>"],    optional
    ["at", "<a place the work goes: file:line, a URL, any text>"],   zero or more
    ["check", "<the command that proves the work>"],                  optional
    ["session", "<the session of a trail>"],                          optional, a trail only (CIP-14)
    ["L", "celis.step"],
    ["l", "step", "celis.step"]
  ],
  "content": "<the work>"
}
```

**Rules**

- A step MUST carry content, the anchor `project:<project id>` and the project name.
- A step MAY carry one `under`, one `fixes`, one `replaces`, one `check`, and any number of `at`.
- A reader MUST count each `file` tag of a step as a place, after its `at` places.
- A reader MUST count a kind 1111 record as a step only when it carries the label `step` in
  `celis.step`. It MUST ignore tags it does not know.
- A reader of decisions MUST NOT count a step as a decision (CIP-10, "Decisions").
- A step's text line MUST hold only its title. The full tree MUST print the other lines of its text under
  it (CIP-14, "The commands").

**Examples**

```
"Write a test: undo restores the cursor (spec/editing.md, \"Undo\")."     at crates/editor/src/undo.rs:40
"Make the onboarding title use the new type scale."                       at https://figma.com/design/Ab12/…
"Round the senior price down\nThe test expects 7.99 for 9.99, not 8.00."   a title and its detail
```

## Projects

A step names a project as CIP-10 "Projects" states. Outside git, the place of the work names it.

```
project id   inside git: as CIP-10 "Projects" states
             outside git, with a first place that is a URL:
               figma:<file key>   a figma.com URL, /file/<key>/… or /design/<key>/…
               web:<host>         any other URL
             else: as CIP-10 "Projects" states
project name the file name in a Figma URL, else the host, else as CIP-10 "Projects" states
```

**Rules**

- `step add` outside git, with a first `--at` that is a URL, MUST name the project of that URL.
- Two steps at the same Figma file MUST name the same project id, whatever the working folder.
- Each step MUST carry the project name its own URL holds. Two steps of one project MAY carry two
  names.

> **Why the place.** A design team never runs `celis` in a repo. Its work sits in a Figma file, and the
> file key stays the same for every person who opens it.

## Starting

A step starts when an agent takes it up. A started step is working until it closes, so the person sees
which step the agent is on. The start states whether the step's check failed before the work.

```json
start  kind 1985, signed by the account (NIP-32)
{ "kind": 1985, "content": "",
  "tags": [["L", "celis.step"], ["l", "start", "celis.step"], ["e", "<step id>"],
           ["before", "fails" | "passes"]] }                    before: only on a step with a check
```

```
open      no `start`, `done` or `drop` names the step
working   a `start` names the step, and no `done` or `drop` does
done      a `done` names the step
dropped   a `drop` names the step
```

**Rules**

- A reader MUST work out a step's state from its `start`, `done` and `drop` records.
- A `start` MUST NOT close its step.
- A new `start` MUST run the step's check, when it has one, and carry its result as `before`.
- When the check passes before the work, `start` MUST say so on stderr and still start the step.
- `tree --json` MUST print the `before` of a started step.

## Closing

A step closes when it is done. A done step carries its proof, and what changed from its plan. What cost
time becomes a decision, so the next agent at that place reads it.

```json
done   kind 1985, signed by an agent (NIP-32)
{ "kind": 1985, "content": "<the proof: the check that failed before and passes now>",
  "tags": [["L", "celis.step"], ["l", "done", "celis.step"], ["e", "<step id>"],
           ["changed", "<how the work differs from the step's title>"]] }      changed: optional
```

**Rules**

- A step MUST be open until a `done` or a `drop` names it. The first of them MUST settle it.
- A reader MUST ignore a `done` record with no content. It does not close the step.
- `done` MUST take `--changed <text>` or `--as-planned`. It MUST refuse neither or both as a usage error.
- `done --changed` MUST carry its text as the `changed` tag. A `done` with no `changed` went as planned.
- `tree --json` MUST print the `changed` of a done step. `tree --full` MUST print it after the proof.

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

## Dropping

A step is dropped when its work is no longer wanted. A drop closes it without a proof, so a trail left
behind or a wrong step leaves the open steps.

```json
drop   kind 1985, signed by an agent (NIP-32)
{ "kind": 1985, "content": "<why the work is no longer wanted>",
  "tags": [["L", "celis.step"], ["l", "drop", "celis.step"], ["e", "<step id>"]] }
```

**Rules**

- A reader MUST ignore a drop with no content. It does not close the step.
- A dropped step MUST NOT have a grade.
- `add --replaces <id>` MUST write the new step under the old step's `under`, when no `--under` is given.
- It MUST then drop the old step and each open step under it, with `replaced by <new id, first 8>`.
- It MUST refuse a trail as a usage error, and a closed step with `closed`.

## Editing

An edit gives a step a new title or text, new places, or a new check. A trail takes more asks as its
session goes on, so its first title stops covering the work; the agent then edits it.

```json
edit   kind 1985, signed by an agent (NIP-32)
{ "kind": 1985, "content": "<a new title, or a title and the detail; may be empty>",
  "tags": [["L", "celis.step"], ["l", "edit", "celis.step"], ["e", "<step id>"],
           ["at", "<a place>"],                                          zero or more
           ["check", "<the command that proves the work>"]] }           optional
```

**Rules**

- A reader MUST apply a step's edits oldest first.
- An edit with one line of content MUST change only the title, and keep the detail.
- An edit with more lines MUST replace the text. One with no content MUST keep the text.
- An edit with `at` tags MUST replace the step's places. One with a `check` MUST replace its check.
- `step add` MUST say when a session's step joins its trail. It MUST print `celis: the step joined the
  trail <id, first 8>; edit its title when it no longer covers the work.` on stderr.

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
celis step add [--under <id>] [--fixes <id>] [--replaces <id>] [--at <place>]... [--check <command>] [--new] -- <text>
                                                               leave a step
celis step start <id>                                         take up a step, and see its decisions
celis step done <id> [--changed <text>] [--as-planned] [--pitfall <text>] -- <proof>
                                                              close a step with its proof
celis step steer <id> -- <text>                                record what the person asked
celis step drop <id> -- <reason>                              close a step whose work is no longer wanted
celis step edit <id> [--at <place>]... [--check <command>] [-- <text>]
                                                              give a step a new title, places or check
```

**Rules**

- Every command MUST take an id as CIP-11 "Output and errors" states.
- A command MUST refuse a start that more than one step's id has, with `no-step`.
- `add` MUST refuse empty text as a usage error.
- `add` MUST refuse an `--under` or a `--fixes` that no step has, with `no-step`. It MUST refuse an
  `--under` step that is closed, with `closed`.
- `add` MUST say on stderr when the title has more than 8 words. It MUST still write the step.
- `add` MUST carry the whole ids of `--under` and `--fixes`, each `--at` and the `--check` as their tags.
  It MUST write each `--at` as CIP-10 "Places" states.
- `add` MUST NOT write a step when an open step of the same project and `under` has the same text. It
  MUST print the id of that step.
- `start` MUST refuse an id no step has, with `no-step`, and a closed step, with `closed`. On a working
  step, it MUST write nothing and print the id of the step's first `start`.
- `start` MUST then print the live decisions at the step's places (CIP-10, "Reading"),
  each once.
- `done` MUST refuse an id no step has, with `no-step`, and a closed step, with `closed`.
- `done` MUST refuse an empty proof, as a usage error.
- `done` with `--pitfall` MUST also write it as a decision on the step's project (CIP-10, "Decisions").
- The scope of that decision MUST be `at:<the step's at>`, else `trail:<id of the step's trail>`.
- `done` MUST refuse a step with an open step under it, with `open-steps`, and write nothing.
- `done` MUST run the step's `check` in the project's top folder, when it has one. When the check fails,
  it MUST refuse with `check-failed`, print the end of the check's output, and write nothing.
- `done` MUST stop a check that runs past `CELIS_CHECK_SECONDS`, 600 when not set, and count it as failed.
- `done` MUST say when a trail's tree holds a multiple of 5 done steps under the trail. It MUST print
  `celis: <n> steps of the trail <id, first 8> are done; check its direction.` on stderr, and exit 0.
- It MUST then name the ask of the step, and each open step of that ask. It MUST ask to drop or
  replace each open step that does not serve the ask.
- When the ask has a check, the direction check MUST run it under the ask's line. It MUST say whether
  it passes, with the last line it printed when it fails.
- `steer` MUST refuse an id no step has, with `no-step`, and a closed step, with `closed`. It MUST refuse
  empty text as a usage error.
- `drop` MUST refuse an id no step has, with `no-step`, and a closed step, with `closed`. It MUST refuse
  an empty reason as a usage error.
- `drop` MUST write a drop for the step and for each open step under it, at any depth.
- `edit` MUST refuse an id no step has, with `no-step`. It MUST refuse an edit with no text, `--at` or
  `--check`, as a usage error. It MUST write each `--at` as CIP-10 "Places" states.
- The JSON of a step MUST carry its `under` and `fixes` as `links`, in that order. A step with neither
  MUST carry an empty `links`.
- `add`, `start`, `done`, `steer`, `drop` and `edit` MUST print the id of each record they wrote, the step's own
  first. With `--json`, each prints `{"id"}`, one a line.

```
text   <id, first 8> [<project name>] <title>               a step
       <id, first 8> [<project name>] <title> · <at>         a step with a place: its first
json   {"id", "created_at", "project", "links": [{"to", "kind": "under | fixes"}], "places": [], "check",
       "content"}
```

**Examples**

```
celis step add --under 3fa2c1d0 --at shelf/prices.py:40 \
  -- "Write a test: a senior pays 80% of the price."
celis step add --under 3fa2c1d0 --fixes 4c1d22e0 -- "Round the senior price down, not up."
celis step add --under 3fa2c1d0 --at https://figma.com/design/Ab12/Shop \
  -- "The price list frame shows the senior tier."
celis step start 4c1d22e0
celis step done 4c1d22e0 --as-planned -- "python3 -m unittest: test_senior failed before, passes now."
celis step drop 3fa2c1d0 -- "The person chose the shared price table instead."
```

## Grades

A grade shows how the work of a closed step went, so a new plan can follow the best earlier path.

```
took      the time from the step's first `start` to the `done` that settled it; with no `start`, from
          the step
rework    the steps whose `fixes` names it
steered   the steers that name it
```

**Rules**

- A reader MUST work out a grade from the records alone. A grade MUST NOT be a record.
- `tree --full` MUST print a done step's grade under it, after its proof.

```
text   took <45s | 1m 30s | 2h 5m | 3d 2h> · rework <n> · steered <n>
json   "grade": {"took": <seconds>, "rework": 0, "steered": 0}
```
