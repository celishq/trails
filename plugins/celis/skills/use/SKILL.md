---
name: use
description: Does an ask with a celis named trail, so the ask keeps the trail's rules. Use when the person runs /celis:use <name> <ask>, or asks to do work with a celis trail by its name.
---

The first word after `/celis:use` is the name of a trail, and the rest is the ask.

1. Run `celis names`. When no trail has the name, show the person the list, and ask which one they
   mean. With no ask, ask what to do.
2. Do the ask as the card says. Add it with `--uses <name>`, and with each other `--uses` it needs.
3. `step start` prints the named trail's rules: keep them. Read its steps with
   `celis tree <its trail id>`, and take those that fit the ask, as you would take past work.
