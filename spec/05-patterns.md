---
cip: 5
title: Patterns
status: draft
depends: [CIP-01]
---

# CIP-05 — Patterns

A pattern says what to choose in a context, and why. It holds in its scope until another replaces it. An
agent writes it when it learns a choice that later work will meet again. A pattern guides: where its
forces are present, work resolves them; where they are not, it does not apply.

## Patterns

```json
{ "kind": 1111,
  "tags": [
    ["I", "project:<project id>"], ["K", "project"],
    ["i", "project:<project id>"], ["k", "project"],
    ["project", "<project name>"],
    ["scope", "me"],
    ["replaces", "<id of the pattern it replaces>"],
    ["path", "<glob>"],
    ["L", "celis.pattern"], ["l", "pattern", "celis.pattern"]
  ],
  "content": "When <the choice and its context>\nForces: <what pulls against what>\nDo: <the resolution>\nBecause: <why it resolves the forces>" }
```

- `scope`: optional; where the pattern holds ("Scope"). With none, it holds in the project.
- `replaces`: optional; the pattern this one takes the place of.
- `path`: optional, repeatable; a file the pattern acts on, as a glob from the project's top folder
  ("Triggers").
- content: text for an agent. A reader MAY show it, and does not parse it.

A drop (CIP-03, "Dropping") MAY name a pattern, with why it no longer holds.

**Writers**

- A writer MUST write a pattern in `celis.pattern`.

**Readers**

- A pattern is live until another pattern names it in `replaces`, or a drop names it.

## Scope

```
project   in its project; a pattern with no `scope` tag
me        in every project
```

**Writers**

- A linked folder MUST NOT hold a `me` pattern (CIP-01, "Links").

**Readers**

- A reader MUST count a pattern with no `scope` tag as `project`, and skip one whose scope it does not
  know.
- A `me` pattern holds in every project. Any other holds only in its own project.

## Reading

**Readers**

- A reader MUST read `decision` in `celis.decision`, which older writers wrote, as `pattern` in
  `celis.pattern`.
- A reader MUST read a pattern with the `binds` label in `celis.pattern` or `celis.decision`, which older
  writers wrote, as a plain pattern.
- A reader MUST ignore a `cell` tag, which older writers wrote.
- A reader MUST read the scope `at:<place>`, which older writers wrote, as `project`, with two more
  `path` globs: the place's path without its line, and that path with `/**` after it. A place of the
  project's top folder, such as `.`, adds no `path`.
- A reader MUST read a live pattern with the scope `trail:<id>`, which older writers wrote, as a note on
  the trail `<id>` (CIP-04, "Notes"), and not as a pattern.

## Triggers

```
<glob>    a path from the project's top folder: * within a segment, ** across segments, ? one character
```

**Readers**

- A pattern matches a file when one of its `path` globs matches the file's path.

## Graph

```json
{ "kind": 1111,
  "tags": [ …, ["within", "<id of the larger pattern>"] ],
  "content": "When the hero of a page is drawn\nForces: …\nDo: …\nBecause: …" }
```

- `within`: optional, repeatable; the larger pattern this one completes. It MAY name a pattern of another
  project.

**Writers**

- A writer MUST write the whole id of each larger pattern, and only of a live pattern the store holds.
- A pattern that replaces another MUST keep each `within` of the one it replaces.

**Readers**

- A reader MUST ignore a `within` that names no record it holds.
- The smaller patterns of a pattern are the live patterns whose `within` names it.

## Examples

```json
{ "kind": 1111,
  "tags": [ …,
    ["good", "<step id>"],
    ["good", "blossom:<sha256>.<ext>?as=<pubkey>&sz=<size>", "<path>"],
    ["bad", "<url>"] ],
  "content": "When a logo is drawn\nForces: …\nDo: keep sharp corners\nBecause: the person picked them" }
```

- `good`: optional, repeatable; an example the pattern shows as better: a done step (CIP-03), a file as
  the Blossom URI of its blob (CIP-08) with its path from the project's top folder, or a URL.
- `bad`: optional, repeatable; an example the pattern shows as worse, in the same forms.

**Writers**

- A writer MUST write the whole id of each step, and only of a step the store holds.
- A writer MUST keep the blob of each file it names.
- A pattern that replaces another MUST keep each example of the one it replaces.

**Readers**

- A reader MUST read a value that starts with `blossom:` as a file, another value with `://` as a URL,
  and else as a step.
