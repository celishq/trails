---
name: create
description: Does a creative ask, such as an image, a logo, a sound, a level or a text, in layers with variants, so the person can pick by taste after. Use when an ask makes something whose quality is a matter of taste.
---

Creative work has no check that proves it good: the person's taste does. Do the ask as the card says,
in layers, with these changes:

1. Before the outer layer, run `celis recall <the file> -- "<the ask>"` and open the files of its
   patterns' examples, to see what the person chose before. In creative work a rule guides too: a
   variant may break it on purpose.
2. The outer layer has 2 variants as steps under it, unless the person names another number: one
   follows the patterns and the past picks, one explores away from them and names the pattern it
   breaks.
   Close each with `--file` for its result, so that its blob stays when the next variant writes the
   same file; when the last one closes, celis prints the blobs beside it.
3. Go as far as you can: choose the variant closer to the good examples, name it in the next layer's
   title, and finish. Only the outer layer has variants, unless the person asks.
4. In the end, name your choice on the "I chose" line, with the blob of each other variant, so the person
   can pick after. When they pick another, keep the pick as a pattern (`steps.md`, "Patterns") and redo
   the next layers from it. A note, such as "move it left", is not a fix of done work.

When the person wants to choose at each layer, `/celis:steer` does one ask that way; when they want it
always, write it as a `me` rule.
