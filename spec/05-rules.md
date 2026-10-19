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
    ["L", "celis.decision"], ["l", "decision", "celis.decision"]
  ],
  "content": "When <what you see>: <what to do>, because <why>." }
```

- `scope`: optional; where the rule holds ("Scope"). With none, it holds in the project.
- `replaces`: optional; the rule this one takes the place of.

**Readers**

- A rule is live until another rule names it in `replaces`.

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
