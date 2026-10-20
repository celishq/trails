---
name: tidy
description: Tidies the celis trails. Drops stale trails and rules, merges rules that say the same thing, settles rules that conflict, and turns work done many times into named trails. Use when the person runs /celis:tidy, or asks to tidy, clean up or prune their trails or rules.
---

You tidy the trails in four parts, in this order: stale, merge, conflicts, templates. Write nothing
before the person says yes.

1. Ask the person what to clean: this project, this project and the `me` rules, or all projects.
2. Add the tidy as one ask, as the card says, and one step under it for each part. Take the parts one
   at a time.
3. For each change, show the person the records it touches, with their ids, and ask: yes, no, or yes to
   all in this part. Write a change only after a yes.
4. To write into another project, add `--project <its id>` to `decide` and `step add`. The id is the
   `project` that `celis tree <a trail of it> --json` gives. `step drop` takes an id of any project.

## Stale

Find what no longer holds:

- a trail that expired, or that is open and no one worked on for weeks;
- an `at:` rule whose place the project no longer has. Check the place only in the working folder's
  project;
- a rule that names code, a file or a command that the project no longer has;
- a rule of a trail whose done work was revised (`revised-by`) when the new work says otherwise.

Drop each one with `celis step drop <id> -- "<why it no longer holds>"`. A dropped trail keeps its
rules: drop each of its rules that is stale too. When you drop a named trail, tell the person that its
name is free.

## Merge

Find live rules of one project and one scope that say the same thing, in any words. When one of them
says it best, keep it, and drop each other with `-- "merged into <its id8>"`. Else write one rule that
says it best with `decide --scope <their scope> --replaces <the newest of them>`, and drop each other
with `-- "merged into <the new id8>"`.

When two or more projects hold the same `project` rule, and it holds for that kind of work anywhere,
offer one `me` rule in place of them.

## Conflicts

Find live rules that hold together, by scope (`me`, `project`, `trail:<id>`, `at:<place>`) or along a
named trail, for the same case, and that say to do different things. Show the person each pair, and ask
which one holds, or for a new rule. Drop the rule that loses with `-- "conflicts with <the winner's
id8>"`. Write a new rule with `decide --replaces <one of them>`, and drop the other.

A rule of a named trail from another project ends only in this project: write the winner with
`decide --replaces <its id>`, as `steps.md` says for named trails.

## Templates

Find groups of 4 or more done trails, of any project, that did the same work in any words. Leave out
named trails and trails dropped after done.

- When `celis names` shows a named trail that does this work, add to it: add an ask under its trail with
  `--project <its project id>`, write each new rule, write each changed rule with `--replaces <its id>`,
  and close the ask.
- Else write a named trail as `steps.md` says, under "Named trails". Give the ask `--uses` with the id
  of each trail of the group. Write the steps that the trails share and the rules that hold for this
  work, in words that hold in any project. Each step's proof is "From <the id8 of each trail>.", and the
  ask's proof is "Made from <n> trails."
- Then offer to drop each `trail:` rule of the group that the named trail now holds, with
  `-- "held by <name>"`.

## End

Close each part's step with what it changed: how many trails and rules it dropped, merged and wrote.
Close the ask. Tell the person what each part changed, and each named trail it wrote.
