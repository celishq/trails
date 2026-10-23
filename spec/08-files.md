---
cip: 8
title: Files
status: draft
depends: [CIP-01, CIP-03]
---

# CIP-08 — Files

A done step can keep the files of its proof, such as an image, a sound or a page. The store keeps each
file once, by its sha256, as a Blossom server does, so a later upload keeps the same name.

## Files of a done

```json
{ "kind": 1985, "content": "<the proof>",
  "tags": [["L", "celis.step"], ["l", "done", "celis.step"], ["e", "<step id>"],
           ["file", "<the file's place>", "<sha256>", "<size>"]] }
```

- `file`: zero or more; a file of the proof: its place when the step closed (CIP-01, "Places"), the
  sha256 of its bytes in lower-case hex, and its size in bytes.

**Writers**

- A writer MUST keep the bytes of each `file` as a blob before it writes the done.

## Blobs

```
$CELIS_HOME/trails/blobs/<sha256>   the bytes of a file, named by their sha256
<folder>/blobs/<sha256>             the same, in a linked folder (CIP-01, "Links")
```

**Writers**

- A blob file MUST NOT change once written, and a reader MUST NOT see it half written.
- While a link is on, a writer MUST write each blob that a `file` tag of a held record names to the
  folder too.

**Readers**

- A reader MUST take a blob whose bytes do not hash to its name as missing.
- While a link is on, a reader MUST copy into the store each blob of the folder that the store lacks.
