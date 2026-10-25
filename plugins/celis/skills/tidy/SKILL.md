---
name: tidy
description: Tidies the celis trails. Drops stale trails and patterns, merges patterns that say the same thing, settles rules that conflict and moves rules into the place's own checks. Use when the person runs /celis:tidy, when celis asks to offer it, or when the person asks to tidy, clean up or prune their trails or patterns.
---

You tidy the trails in four parts, in this order: stale, merge, conflicts, moves. A rule is a pattern
whose grep fields say ` binds`.

1. Ask the person what to clean: this project, this project and the `me` patterns, or all projects.
2. Add one ask titled `Tidy the celis trails`, so that celis knows when the last tidy was, and one step
   under it for each part. Take the parts one at a time.
3. Read the store once: `celis grep -E " (trail|pattern) [0-9a-f]{8} "` gives each trail and pattern
   line. A pattern's fields say ` trigger`, ` binds` and `recalled:<day>`.
4. For each change, show the person the records it touches, with their ids, and ask: yes, no, or yes to
   all in this part. Write nothing before a yes.
5. To write into another project, add `--project <its id>` to `pattern` and `step add`; the id is the
   `project` that `celis tree <a trail of it> --json` gives.

## Stale

Drop with `celis step drop <id> -- "<why>"`:

- a trail that expired, or that is open and untouched for weeks;
- a pattern that names a place, code or a command the project no longer has;
- a pattern of revised work (`revised-by`) that the new work contradicts;
- a one-off note about one event;
- a pattern older than 60 days with no `recalled:<day>` in the last 60 days.

A dropped trail keeps its patterns: drop each that is stale too.

## Merge

Find live patterns of one project and scope that say the same thing, or whose good examples make one
point. Keep the one that says it best, or write one in the four lines with `pattern --replaces <the
newest>`; drop each other with `-- "merged into <id8>"`. A merged pattern gives `--good` and `--bad`
again for the examples of each pattern it does not replace, and `--binds` when one of them binds. When
two or more projects hold the same `project` pattern, offer one `me` pattern in its place.

## Conflicts

Find live patterns that hold together for the same case and say to do different things. When both
guide, a pattern may name why it wins in its `Forces`; when a rule is one of them, or both bind, ask the
person which holds, or for a new one, and drop the loser with `-- "conflicts with <id8>"`.

## Moves

Find live rules that the place can hold itself: a check of a tool, such as the linter, the tests, CI or
a Figma project's styles, or else a line in the project's agent file, `AGENTS.md` when it has none.
After a yes, make the change and run the check. Then drop the rule with `-- "<the check> holds it"`.

## End

Close each part's step with how many trails and patterns it dropped, merged, moved and wrote, and close
the ask. Tell the person what each part changed.
