celis keeps each ask as a trail of steps, and the rules that later work must keep.

```
celis decide [--scope at:<place> | trail:<id> | me] [--replaces <id>] -- "<rule>"
celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--at <place>] [--check <command>] [--new] [--session <id>] -- "<title>"
celis step start <id>
celis step done <id> [--changed "<what differs>"] [--as-planned] [--pitfall "<what cost time>"] -- "<proof>"
celis step drop <id> -- "<why>"
celis step edit <id> [--at <place>]... [--check <command>] [-- "<new title>"]
celis tree [<id>] [--full] [--all] [--demo]
```

1. When celis asks for the `celis:past` agent, run it as celis says, even for a simple ask. With no such
   agent, do what the `past.md` the session start names says, yourself. When the ask conflicts with a
   rule it brings, the ask wins: replace the rule with `decide --replaces <id>`.
2. An ask that changes a file, however small, or that works toward a result someone can check, gets a
   trail. A question, an explanation, a review or an investigation gets none: answer it, and run no
   command below.
3. Add the ask with `step add` where the prompt hook says. With no hint, add it under the session's
   trail, else with `--new`. An ask that corrects done work takes `--replaces <its id>`. Give the ask the
   project's own check, and `--uses <id>` for each decision or past work it follows.
4. For a larger ask, add 2 to 5 steps under it with a title only, one for each big shape of the work.
   Break down the shape with the most doubt, or that the others need, one level at a time, until a step
   is one action: one edit, one command or one read. Only that step gets `--at`, a `--check` that proves
   it, and `expect: <the result>` on the line after its title. Add the next step after you close this one.
5. For each step: `start`, and keep the decisions it prints. Do the work, then `done` with a proof a
   person can check: `--as-planned` when the result matches `expect`, else `--changed`. Add `--pitfall`
   when something cost time. A wrong shape, or a fix to done work, takes `--replaces <id>`.
6. When a step grows past one action, or you try the same thing twice: stop. Write what you learned with
   `decide`, split the step, and take the first part.
7. Do what celis prints after each command. It names the next move.
8. Close the ask, never the trail. Tell the person each rule you wrote.

A rule reads "When <what you see>: <what to do>, because <the reason>." Its scope is `trail:<id>` when it
holds for this task, `project` (the default) when it holds for the repo, and `me` when it holds in all the
person's work. When the person asks you to remember something, write it as one rule. Read the `steps.md`
the session start names for moved files, past work, subagents, links, and when to stop.
