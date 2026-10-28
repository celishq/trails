---
cip: 8
title: Files
status: draft
depends: [CIP-01, CIP-03, CIP-05]
---

# CIP-08 — Files

A done step can keep the files of its proof, such as an image, a sound or a page. The done describes each
file as NIP-92 and BUD-10 do, so relays and Blossom servers read the record as it is. The store keeps each
file once, by its sha256, as a Blossom server does, so a later upload keeps the same name.

## Files of a done

```json
{ "kind": 1985,
  "content": "<the proof>\nblossom:<sha256>.<ext>?as=<pubkey>&sz=<size>",
  "tags": [["L", "celis.step"], ["l", "done", "celis.step"], ["e", "<step id>"],
           ["imeta", "url blossom:<sha256>.<ext>?as=<pubkey>&sz=<size>", "x <sha256>",
            "m <media type>", "size <size>", "path <the file's place>"]] }
```

- `imeta`: zero or more; a file of the proof, as NIP-92 gives it:
  - `url`: its Blossom URI (BUD-10): the sha256, the extension of its path or of its media type, else
    `bin`, the hex pubkey of the account that signs, and the size.
  - `x`: the sha256 of its bytes, in lower-case hex.
  - `m`: its media type: by its extension, else as its `data:` URI gives it, else by its first bytes;
    `application/octet-stream` when none gives one.
  - `size`: its size in bytes.
  - `path`: its place when the step closed (CIP-01, "Places"); none for bytes of stdin or a `data:`
    URI.
- content: the proof, then the `url` of each file, one a line.

**Writers**

- A writer MUST keep the bytes of each file as a blob before it writes the done.

**Readers**

- A reader MUST read a file only from an `imeta` with `x`; it MAY fetch a blob that the store lacks
  from the servers that the `url` names.

## Blobs

```
$CELIS_HOME/trails/blobs/<sha256>   the bytes of a file, named by their sha256
<folder>/blobs/<sha256>             the same, in a linked folder (CIP-01, "Links")
```

**Writers**

- A blob file MUST NOT change once written, and a reader MUST NOT see it half written.
- While a link is on, a writer MUST write each blob that an `imeta` or a file example (CIP-05,
  "Examples") of a held record names to the folder too.

**Readers**

- A reader MUST take a blob whose bytes do not hash to its name as missing.
- While a link is on, a reader MUST copy into the store each blob of the folder that the store lacks.
