celis keeps each ask as a trail: its steps, their proofs, and the rules that later work must keep.

```
celis decide [--scope at:<place> | trail:<id> | me] [--replaces <id>] -- "<rule>"
celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--at <place>] [--check <command>] [--new] [--session <id>] -- "<title>"
celis step start <id>
celis step done <id> [--changed "<what differs>"] [--as-planned] [--pitfall "<what went wrong>"] -- "<proof>"
celis step drop <id> -- "<why>"
celis step edit <id> [--at <place>]... [--check <command>] [-- "<new title>"]
celis tree [<id>] [--full] [--all] [--demo]
```

1. When celis asks for the `celis:past` agent, run it as celis says, even for a simple ask. With no such
   agent, do what the `past.md` the session start names says, yourself. When the ask conflicts with a
   rule it brings, the ask wins: replace the rule with `decide --replaces <id>`.
2. An ask that changes a file, however small, or that works toward a result someone can check, gets a
   trail: review feedback to fix too. A question, an explanation, or a review or investigation you only
   report on gets none: answer it, and run no command below.
3. Add the ask with `step add` where the prompt hook says. With no hint, add it under the session's
   trail, else with `--new`. An ask that corrects done work takes `--replaces <its id>`. Give the ask the
   project's own check, and `--uses <id>` for each decision or past work it follows.
4. For a larger ask, add 2 to 5 steps under it with a title only. A title says what is true when the step
   is done, not how to do it, so that a person can check it: "The cart total is correct", not "Sum the
   cart in cents". The steps under a step say how. Split first the step with the most doubt, or the step
   that others need, one level at a time, until a step is one action: one edit, one command or one read.
   Only that step gets `--at`, a `--check` that proves it, and `expect: <the result>` on the line after
   its title. Add the next step after you close this one.
5. For each step: `start`, and keep the decisions it prints. Do the work, then `done` with a proof a
   person can check: `--as-planned` when the result matches `expect`, else `--changed`. Add `--pitfall`
   when something went wrong on the way. A wrong step, or a fix to done work, takes `--replaces <id>`.
6. When a step grows past one action, or you try the same thing twice: stop. Write what you learned with
   `decide`, split the step, and take the first part.
7. Do what celis prints after each command. It names the next move.
8. Close the ask, never the trail. Tell the person about the ask by the steps right under it, and each
   rule you wrote.

A rule reads "When <what you see>: <what to do>, because <the reason>." Its scope is `trail:<id>` when it
holds for this trail, `project` (the default) when it holds for the repo, and `me` when it holds in all the
person's work. When the person asks you to remember something, or states how the work must be done
(review feedback, a convention, a correction), write each point as one rule before you do the work.
Read the `steps.md` the session start names for moved files, past work, subagents, links, and when to
stop.
