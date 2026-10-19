celis keeps each ask as a trail: its steps, their proofs, and the rules that later work must keep.

```
celis decide [--scope at:<place> | trail:<id> | me] [--replaces <id>] [--project <id>] -- "<rule>"
celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--at <place>] [--check <command>] [--new] [--session <id>] [--project <id>] -- "<title>"
celis step start <id>
celis step done <id> [--changed "<what differs>"] [--as-planned] [--pitfall "<rule>"] -- "<proof>"
celis step drop <id> -- "<why>"
celis step edit <id> [--at <place>]... [--check <command>] [-- "<new title>"]
celis tree [<id>] [--graph] [--full] [--all] [--demo]
```

1. When celis asks for the `celis:past` agent, run it as celis says, even for a simple ask. With no such
   agent, do what the `past.md` the session start names says, yourself. When the ask conflicts with a
   rule it brings, the ask wins: replace the rule with `decide --replaces <id>`.
2. An ask that changes a file, however small, or that works toward a result someone can check, gets a
   trail: review feedback to fix too. A question, an explanation, or a review or investigation you only
   report on gets none: answer it, and run no command below.
3. Add the ask with `step add` where the prompt hook says. With no hint, add it under the session's
   trail, else with `--new`. An ask that corrects done work takes `--replaces <its id>`. Give the ask the
   project's own check, and `--uses <id>` for each rule or past step it follows.
4. A step's title names what the person wants done: a verb and what it acts on, such as "Split a long
   file into its parts". Use words that hold in another project, and name the case only when that helps
   the next agent. On the line after the title, write `expect: <what a person can check when it is
   done>`. The steps under a step say how to do it. For a larger ask, add 2 to 5 steps under it. Split
   first the step with the most doubt, or the step that others need, one level at a time, until a step
   is one action: one edit, one command or one read. Only that step gets `--at` and a `--check` that
   proves it. Add the next step after you close this one.
5. For each step: `start`, and keep the rules it prints. Do the work, then `done` with a proof a
   person can check: `--as-planned` when the result matches `expect`, else `--changed`. When something
   went wrong, add `--pitfall` with the rule that keeps it from happening again. A wrong step, or a fix
   to done work, takes `--replaces <id>`.
6. When a step grows past one action, or you try the same thing twice: stop. Write what you learned with
   `decide`, split the step, and take the first part.
7. Do what celis prints after each command. It names the next move.
8. Close the ask, never the trail. Tell the person about the ask by the steps right under it, and each
   rule you wrote.

A rule reads "When <what you see>: <what to do>, because <the reason>." Its scope is `trail:<id>` when it
holds for this trail, `project` (the default) for the repo, and `me` for all the person's work. When the person asks you to remember something, or states how the work must be done
(review feedback, a convention, a correction), write each point as one rule before you do the work.
Read the `steps.md` the session start names for moved files, past work, subagents, links, and when to
stop.
