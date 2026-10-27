---
cip: 9
title: Cells
status: draft
depends: [CIP-01, CIP-05, CIP-08]
---

# CIP-09 — Cells

A cell is a named set of patterns of one account. A project turns a cell on or off, and the view of its
patterns follows. A cell goes to another account as a folder of its records and blobs.

## Membership

```json
{ "kind": 1985, "content": "",
  "tags": [["L", "celis.cell"], ["l", "eating-east", "celis.cell"],
           ["e", "<id of a pattern>"], ["e", "<id of a pattern>"]] }
```

- `l`: the cell's name: 1 to 64 characters of `a-z`, `0-9` and `-`, starting with a letter or a digit.
- `e`: one or more; a pattern the membership puts in the cell.

A cell is its name and the account that signs its memberships. The cell's patterns are the patterns its
memberships name, and each pattern of the same account that replaces one of the cell's patterns.

**Writers**

- A writer MUST write the whole id, and only of a live pattern that the same account signed.
- A writer MUST NOT write a membership of a name that a membership of another account in the store
  holds.

**Readers**

- A reader MUST ignore an `e` that names a pattern another account signed, or no record it holds.
- Two accounts that write one name make two cells.

## Masks

```json
{ "kind": 1985, "content": "",
  "tags": [["L", "celis.mask"], ["l", "on", "celis.mask"],
           ["cell", "eating-east"], ["p", "<hex pubkey of the cell's account>"],
           ["I", "project:<project id>"], ["K", "project"],
           ["i", "project:<project id>"], ["k", "project"],
           ["project", "<project name>"]] }
```

- `l`: `on` or `off`.
- `cell` · `p`: the cell's name and account.
- The project anchor (CIP-01, "Records"), for a mask of one project; or `["scope", "me"]` and no anchor,
  for a mask of every project.

**Writers**

- A writer MUST write a mask with a `created_at` greater than that of each mask of the same cell and
  the same project, or of every project, that the store holds.

**Readers**

- In a project, a cell is on or off as its newest mask of that project gives it, else as its newest `me`
  mask gives it, else off. The newest mask has the greatest `created_at`, then the greatest id.
- A reader MUST ignore a mask whose `l` is not `on` or `off`.

## The view

**Readers**

- In a project, a pattern is out of the view when a cell holds it and each cell that holds it is off, or
  when its `replaces` names a pattern that is out of the view. Else it is in the view.
- A pattern that is out of the view is not in scope (CIP-05, "Scope").
- A live pattern of a cell that is on is in scope in the project, whatever its `scope` and `path` tags.
- A reader MUST ignore a `within` (CIP-05, "Graph") that names a pattern out of the view.

## Export

```
<folder>/
  cell.jsonl              one record a line, as NIP-01 JSON
  blobs/<sha256>.<ext>    a blob (CIP-08, "Blobs"), with the extension of its Blossom URI
  cell.md                 the patterns as text for a person
```

**Writers**

- `cell.jsonl` MUST hold the live patterns of the cell, then one membership, signed by the cell's account,
  that names each of them. It MUST NOT hold another record.
- The folder MUST hold the blob of each file example (CIP-05, "Examples") of the patterns.

**Readers**

- A reader MUST read only `cell.jsonl` and `blobs/`.
- A reader MUST refuse the whole folder, with `invalid` (CIP-02), when a line does not parse or does
  not verify, when a record is not a pattern or a membership, when there is not one membership, or when
  a pattern is not named by it or is signed by another account.
- A reader MUST take a blob only when its bytes hash to its name (CIP-08, "Blobs"), and else take it as
  missing.
