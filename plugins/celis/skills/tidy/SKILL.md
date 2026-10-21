---
name: tidy
description: Tidies the celis trails. Drops stale trails and rules, fades rules no one corrects, fixes triggers that miss, merges rules that say the same thing, settles rules that conflict, moves rules into the place's own checks, and turns work done many times into named trails. Use when the person runs /celis:tidy, or asks to tidy, clean up or prune their trails or rules.
---

You tidy the trails in six parts, in this order: stale, log, merge, conflicts, moves, templates. Write
nothing before the person says yes.

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
- a rule of a trail whose done work was revised (`revised-by`) when the new work says otherwise;
- a one-off note: a rule about one event, not about work to come.

Drop each one with `celis step drop <id> -- "<why it no longer holds>"`. A dropped trail keeps its
rules: drop each of its rules that is stale too. When you drop a named trail, tell the person that its
name is free.

## Log

`$CELIS_HOME/given` has a JSON line for each rule given in a session, and a line with `"miss": true` for
each rule an agent replaced in a session that did not give it. Count the lines of each rule. Off macOS,
the default of `$CELIS_HOME` is `$HOME/.local/share/celis`:

```bash
jq -r 'select(.rule) | "\(.rule[:8]) \(if .miss then "miss" else "given" end)"' "${CELIS_HOME:-$HOME/Library/Application Support/Celis}/given" | sort | uniq -c
```

- Fade a rule given 10 times or more that has no miss and that no rule replaces: drop it with
  `-- "faded: given <n> times, never corrected"`.
- For each rule with a miss, find the work of the miss's session: the `step` of the session's other
  lines, and `celis tree <step>`. Run `celis rules --all <a file of that work>`, or `-- "<its
  command>"`: `--all` only looks, and logs nothing. When the rule shows, the cap cut it: leave it. Else its trigger
  missed: write the same text with a trigger that matches work like that, with `decide --replaces <its
  id> --scope <its scope>` and `--path`, `--content`, `--command` or `--tool`.

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

## Moves

Find live rules that the place can hold itself. Leave out rules that grep marks ` promoted`.

- A rule that a tool of the place can check: propose the change to that tool, such as the repo's
  linter, tests or CI, or a Figma project's styles.
- A rule that holds for all work in the project, that no tool can check: propose a line in the project's
  agent instruction file, `AGENTS.md` when it has none.

Show the person the rule and the change. After a yes, make the change, and run the check when there is
one. Then mark the rule with `decide --replaces <its id> --scope <its scope> --promoted -- "<the same
text>"`: celis keeps it, and knows that the place checks it.

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

Close each part's step with what it changed: how many trails and rules it dropped, faded, merged, moved
and wrote. Close the ask. Tell the person what each part changed, and each named trail it wrote.
