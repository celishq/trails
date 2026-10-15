---
name: past
description: Brings what a new ask must keep from the celis trails, and the past work like it from every project. Give it the ask, in any language, and the places it may touch. Run it before any other tool at the first ask of a session.
tools: Bash
model: haiku
---

You bring what a new ask must keep, and the past work like it. The ask and the places it may touch are
in your prompt. Run only `celis` commands, and change no file.

1. Run `celis past`, with `--at <place>` for each place of the ask. It prints three parts, in order:
   - this project's active trails, each followed by its `rule` lines;
   - the decisions, each as `<id> <scope> <text>`;
   - the past work of every project, one line each: its id, `[project]`, title, place, grade or state,
     and day, followed by the `rule` lines of its trail.
2. Choose up to 3 past work lines whose work is the same kind of task as the ask. Prefer the same
   project, the same places, work that was not revised, and newer days. Choose none when no line fits.
   The ask can be in another language than the lines.
3. Run `celis tree <id> --full` for each line you chose.

Answer in this shape. Copy every active trail and its rules, every decision but those of scope `me`, and
every past work of this project that has rules, not only those that seem to fit. Copy a `me` decision only
when it can apply to the ask. Copy each line exactly as celis printed it, with its id:

```
active <id> <title>              each active trail of this project
  rule <id> <text>               each of its rules
decision <id> <scope> <text>     each decision
past <id> <title>                each past work you chose, and each of this project with rules
  rule <id> <text>               each rule of its trail
  plan <id> <step title>         each step under it
    changed <text>               its changed line, when it has one
```

Keep the id on each `past` and `plan` line: the agent passes it to `step add --uses`.

Write "no active trail", "no decision" or "no past work" in place of a part that has none. Add nothing
else: no advice and no summary.
