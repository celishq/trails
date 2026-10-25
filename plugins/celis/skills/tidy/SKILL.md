---
name: tidy
description: Tidies the celis trails. Drops stale trails, rules and patterns, merges rules and patterns that say the same thing, turns patterns that make one point into a rule, settles rules that conflict and moves rules into the place's own checks. Use when the person runs /celis:tidy, when celis asks to offer it, or when the person asks to tidy, clean up or prune their trails, rules or patterns.
---

You tidy the trails in five parts, in this order: stale, merge, patterns, conflicts, moves.

1. Ask the person what to clean: this project, this project and the `me` rules, or all projects.
2. Add one ask titled `Tidy the celis trails`, so that celis knows when the last tidy was, and one step
   under it for each part. Take the parts one at a time.
3. Read the store once: `celis grep -E " (trail|rule) [0-9a-f]{8} "` gives each trail and rule line. A
   rule's fields say ` pattern`, ` promoted` and `recalled:<day>`.
4. For each change, show the person the records it touches, with their ids, and ask: yes, no, or yes to
   all in this part. Write nothing before a yes.
5. To write into another project, add `--project <its id>` to `decide` and `step add`; the id is the
   `project` that `celis tree <a trail of it> --json` gives.

## Stale

Drop with `celis step drop <id> -- "<why>"`:

- a trail that expired, or that is open and untouched for weeks;
- a rule that names a place, code or a command the project no longer has;
- a rule of revised work (`revised-by`) that the new work contradicts;
- a one-off note about one event;
- a rule or a pattern older than 60 days with no `recalled:<day>` in the last 60 days.

A dropped trail keeps its rules: drop each that is stale too.

## Merge

Find live rules or patterns of one project and scope that say the same thing. Keep the one that says it
best, or write one with `decide --replaces <the newest>`; drop each other with `-- "merged into <id8>"`.
A merged pattern gives `--good` and `--bad` again for the examples of each pattern it does not replace.
When two or more projects hold the same `project` rule, offer one `me` rule in their place.

## Patterns

Find 3 or more live patterns of one scope whose good examples make one point. Propose one rule that says
it; after a yes, write it, and drop each pattern with `-- "made the rule <id8>"`, unless its examples
show more than the rule.

## Conflicts

Find live rules that hold together for the same case and say to do different things. Ask the person
which holds, or for a new rule; drop the loser with `-- "conflicts with <id8>"`.

## Moves

Find live rules, not ` promoted`, that the place can hold itself: a check of a tool, such as the linter,
the tests, CI or a Figma project's styles, or else a line in the project's agent file, `AGENTS.md` when
it has none. After a yes, make the change and run the check. Then mark the rule with
`decide --replaces <its id> --promoted -- "<the same text>"`, with its scope and paths.

## End

Close each part's step with how many trails and rules it dropped, merged, moved and wrote, and close
the ask. Tell the person what each part changed.
