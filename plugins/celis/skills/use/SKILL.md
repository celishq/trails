---
name: use
description: Does an ask with a celis skill, so the ask keeps the skill's rules. Use when the person runs /celis:use <name> <ask>, or asks to do work with a celis skill by its name.
---

The first word after `/celis:use` is the name of a skill, and the rest is the ask.

1. Run `celis skills`. When no skill has the name, show the person the list, and ask which one they
   mean. With no ask, ask what to do.
2. Do the ask as the card says. Add it with `--uses <name>`, and with each other `--uses` it needs.
3. `step start` prints the skill's rules: keep them. Read its steps with `celis tree <its trail id>`,
   and take those that fit the ask, as you would take past work.
