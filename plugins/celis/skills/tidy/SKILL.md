---
name: tidy
description: Tidies the celis trails. Drops stale trails, rules and patterns, merges rules and patterns that say the same thing, turns patterns that make one point into a rule, settles rules that conflict and moves rules into the place's own checks. Use when the person runs /celis:tidy, when celis asks to offer it, or when the person asks to tidy, clean up or prune their trails, rules or patterns.
---

You tidy the trails in five parts, in this order: stale, merge, patterns, conflicts, moves. Write
nothing before the person says yes. A pattern is a rule whose grep line says ` pattern`.

1. Ask the person what to clean: this project, this project and the `me` rules, or all projects.
2. Add the tidy as one ask titled `Tidy the celis trails`, as the card says, so that celis knows when
   the last tidy was, and one step under it for each part. Take the parts one
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
- a one-off note: a rule about one event, not about work to come;
- a rule or a pattern older than 60 days whose grep line has no `recalled:<day>`, or a day older than
  60 days: no ask has needed it for a long time.

Drop each one with `celis step drop <id> -- "<why it no longer holds>"`. A dropped trail keeps its
rules: drop each of its rules that is stale too.

## Merge

Find live rules of one project and one scope that say the same thing, in any words. When one of them
says it best, keep it, and drop each other with `-- "merged into <its id8>"`. Else write one rule that
says it best with `decide --scope <their scope> --replaces <the newest of them>`, and drop each other
with `-- "merged into <the new id8>"`.

Two patterns that say the same thing merge the same way: the new one gives `--good` and `--bad` again
for each example of the patterns it does not replace, and keeps those of the one it replaces.

When two or more projects hold the same `project` rule, and it holds for that kind of work anywhere,
offer one `me` rule in place of them.

## Patterns

Find 3 or more live patterns of one scope whose good examples make one point, in any words. Propose one
rule that says the point. After a yes, write it with `decide --scope <their scope>`, and drop each
pattern with `-- "made the rule <the new id8>"`, or keep a pattern whose examples show more than the
rule says.

## Conflicts

Find live rules that hold together, by scope (`me`, `project`, `trail:<id>`, `at:<place>`), for the same
case, and that say to do different things. Show the person each pair, and ask
which one holds, or for a new rule. Drop the rule that loses with `-- "conflicts with <the winner's
id8>"`. Write a new rule with `decide --replaces <one of them>`, and drop the other.

## Moves

Find live rules that the place can hold itself. Leave out rules that grep marks ` promoted`.

- A rule that a tool of the place can check: propose the change to that tool, such as the repo's
  linter, tests or CI, or a Figma project's styles.
- A rule that holds for all work in the project, that no tool can check: propose a line in the project's
  agent instruction file, `AGENTS.md` when it has none.

Show the person the rule and the change. After a yes, make the change, and run the check when there is
one. Then mark the rule with `decide --replaces <its id> --scope <its scope> --promoted -- "<the same
text>"`: celis keeps it, and knows that the place checks it.

## End

Close each part's step with what it changed: how many trails and rules it dropped, faded, merged, moved
and wrote. Close the ask. Tell the person what each part changed.
