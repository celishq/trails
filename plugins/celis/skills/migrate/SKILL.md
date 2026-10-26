---
name: migrate
description: Rewrites the stored celis patterns into the four lines When, Forces, Do and Because. Use when the person runs /celis:migrate, or asks to move their old rules to patterns.
---

You rewrite each live pattern in the old one-line form into the four lines that the card shows. Write
nothing to the store before the person says yes.

1. Ask the person what to migrate: this project, this project and the `me` patterns, or all projects.
2. Add the migrate as one ask, as the card says.
3. List the live patterns with `celis grep -E " pattern [0-9a-f]{8} live "`. Skip each whose text has a
   `Forces:` line. When a text does not say enough, read its trail with `celis tree <trail id8>`.
4. For each other pattern, draft its four lines from its text: the `When` from the part before the first
   `:`, the `Do` from what it says to do, the `Because` from its reason, and the `Forces` from what the
   reason pulls against. Keep the meaning, and add no point the old text does not make.
5. Show the person the whole list: each id8, the old text and the four lines. Ask:
   yes, no, or a change. With more than 50 patterns, show and write them in batches of 50.
6. After a yes, write each one with `celis pattern --replaces <id> [--scope <its scope>] [--path <glob>]...
   -- "<the four lines>"`. Keep its scope, a done trail's too, and each `path` that
   `celis tree <its trail> --json` or its grep line shows; `{project}` takes no `--scope`. Its examples
   stay. To write into another project, add `--project <its id>`.
7. Run step 3 again: no pattern must be left. Close the ask with the counts, and tell them to the person:
   patterns rewritten, and patterns left as they were.
