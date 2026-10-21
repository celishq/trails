---
cip: 5
title: Rules
status: draft
depends: [CIP-01]
---

# CIP-05 — Rules

A rule is one sentence that holds in its scope until another replaces it. An agent writes it when it
learns what later work must keep.

## Rules

```json
{ "kind": 1111,
  "tags": [
    ["I", "project:<project id>"], ["K", "project"],
    ["i", "project:<project id>"], ["k", "project"],
    ["project", "<project name>"],
    ["scope", "<me | trail:<step id> | at:<place>>"],
    ["replaces", "<id of the rule it replaces>"],
    ["path", "<glob>"], ["content", "<regex>"], ["command", "<regex>"],
    ["L", "celis.decision"], ["l", "decision", "celis.decision"],
    ["l", "situation", "celis.decision"], ["l", "promoted", "celis.decision"]
  ],
  "content": "When <what you see>: <what to do>, because <why>." }
```

- `scope`: optional; where the rule holds ("Scope"). With none, it holds in the project.
- `replaces`: optional; the rule this one takes the place of.
- `path`: optional, repeatable; a file the rule acts on, as a glob from the project's top folder
  ("Triggers").
- `content`: optional; a regex that the file's text must match.
- `command`: optional; a regex of the shell commands the rule acts on.
- `situation`: optional label; the rule also holds in a case that no call shows ahead.
- `promoted`: optional label; a tool of the place now checks the rule.

`path`, `content` and `command` are the rule's trigger.

A drop (CIP-03, "Dropping") MAY name a rule, with why it no longer holds.

**Readers**

- A rule is live until another rule names it in `replaces`, or a drop names it.

## Scope

```
at:<place>        at one place of its project (CIP-01, "Places")
trail:<step id>   in its project, with the trail, while it is active and after it ends (CIP-04)
project           in its project; a rule with no `scope` tag
me                in every project
```

**Writers**

- A writer MUST write the whole id of the trail that `trail:<id>` names.
- A linked folder MUST NOT hold a `me` rule (CIP-01, "Links").

**Readers**

- A reader MUST count a rule with no `scope` tag as `project`, and skip one whose scope it does not
  know.
- A `me` rule holds in every project. Any other holds only in its own project.

## Reading

**Readers**

- A rule `at:<place>` holds at that place whatever line the place names, under it when it is a
  folder, and at each place under a folder it names.

## Triggers

```
<glob>    a path from the project's top folder: * within a segment, ** across segments, ? one character
<regex>   the syntax of the Rust regex crate: no lookaround and no backreferences
```

**Readers**

- On a file call, a rule matches when one of its `path` globs matches the file and its `content`, when
  set, matches the file's text: the new text on a write or an edit, the file's text on a read. A file
  that does not exist yet matches when one of its `path` globs matches it and the rule has no
  `content`.
- On a shell command, a rule matches when its `command` matches the command, or when a file that the
  command names matches as a file call whose text is the command.
- A rule `at:<place>` counts as a rule with that place as its `path`, which matches as "Reading" states.
- A pattern that does not compile matches nothing.
