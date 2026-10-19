---
cip: 1
title: Records
status: draft
depends: []
---

# CIP-01 — Records

Every trail, step and rule is a nostr record in one store on the machine. This CIP defines the
record, the store, the account that signs, the project a record names, its places, and the folders a
project links.

## Records

A record is a NIP-01 event. A step or a rule is a NIP-22 comment on its project (kind 1111). A start,
done, drop, edit or prompt is a NIP-32 label on a step (kind 1985). Each carries the label namespace of its
CIP.

```json
{ "kind": 1111,
  "tags": [
    ["I", "project:<project id>"], ["K", "project"],
    ["i", "project:<project id>"], ["k", "project"],
    ["project", "<project name>"],
    ["L", "<namespace>"], ["l", "<label>", "<namespace>"]
  ],
  "content": "<text>" }
```

- `I` `K` `i` `k`: the project anchor, as NIP-22 gives it.
- `project`: the project's name.
- `L` `l`: what the record is: `decision` in `celis.decision`, or a label in `celis.step`.

**Writers**

- A comment MUST carry content, the project anchor and the project name.

**Readers**

- A reader MUST ignore a kind, label or tag it does not know, and each deprecated name of the README.

## The store

```
$CELIS_HOME/trails/
  shared/<created_at>-<id>.json    one record, as NIP-01 JSON
```

**Writers**

- A writer MUST write each record to its own file, `<created_at>-<id>.json`, in `$CELIS_HOME/trails/`,
  whatever the working folder. It MUST create the store when there is none.
- A record file MUST NOT change once written, and a reader MUST NOT see it half written.

**Readers**

- A reader MUST skip a file that does not parse, or whose record does not verify.
- A reader MUST read no store as an empty one, and create nothing.

## The account

```
$CELIS_HOME/phrase    12 BIP-39 words, readable by its owner alone
```

The account is the key the phrase gives (`celis-nostr`). It is the key the Celis app gives for the same
phrase.

**Writers**

- A writer MUST sign every record as the account.
- A writer MUST mint the phrase only when it writes a record and none exists. Two writers that mint at
  once MUST end with one phrase.
- A record MUST NOT carry the phrase or the account's secret.

**Readers**

- A reader MUST count every record that verifies, whatever account signed it.

## Projects

```
project id     inside git: the first of `git rev-list --max-parents=0 HEAD`, sorted
               outside git, when the record's first place is a URL:
                 figma:<file key>   a figma.com URL, /file/<key>/… or /design/<key>/…
                 web:<host>         any other URL
               else: the path of the repo's or worktree's top folder, else of the working folder
project name   the Figma file's name, else the host, else the main checkout's folder, else the
               working folder's
```

**Writers**

- Every clone and worktree of one repo MUST give the same project id, and so MUST two records at the same
  Figma file.
- The name MUST NOT change the id. Two records of one project MAY carry two names.

## Places

A place is a file, a `file:line`, a folder or a URL.

**Writers**

- A writer MUST write a path from the project's top folder, resolved against the working folder, with no
  `.` or `..`. A `:<line>` MUST stay at its end.
- A writer MUST keep a URL, and a path outside the top folder, as it was given.

**Examples**

```
in ~/code/shop/src:   ./prices.py:40                         →   src/prices.py:40
in ~/code/shop:       /Users/me/code/shop/src/prices.py      →   src/prices.py
```

## Links

A project can keep a copy of its records in a folder: `.celis/trails/` in the repo, so git carries them
to other clones, or any folder, such as a synced drive.

```
$CELIS_HOME/links                   one link a line: `<project id>\t<folder>`
<folder>/<created_at>-<id>.json     one record of the project
```

- `folder`: a path from the project's top folder when it lies in it, else a full path. Default:
  `.celis/trails`.
- A folder **holds** a record that names the project and has no `me` scope, and a record that names no
  project and names, by an `e` tag, a record the folder holds.
- A link is **on** when `$CELIS_HOME/links` names the project and the folder, and the folder exists in
  the working folder's checkout.

**Writers**

- Only `celis link` and `celis unlink` (CIP-02) MAY write `$CELIS_HOME/links`. A project MAY have more than
  one link.
- While a link is on, a writer MUST write each record the folder holds to the folder too.

**Readers**

- While a link is on, a reader MUST read its folder with the store, and copy into the store each record
  the store lacks. While it is not on, a reader MUST NOT read the folder.
