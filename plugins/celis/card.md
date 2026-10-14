celis keeps each ask of this session as a trail of steps, with the rules that later work must keep.

```
celis decide [--scope at:<place> | trail:<id> | me] [--replaces <id>] -- "<rule>"
celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--at <place>] [--check <command>] [--new] [--session <id>] -- "<title>"
celis step start <id>
celis step done <id> [--changed "<what differs>"] [--as-planned] [--pitfall "<what cost time>"] -- "<proof>"
celis step drop <id> -- "<why>"
celis step edit <id> [--at <place>]... [--check <command>] [-- "<new title>"]
celis tree [<id>] [--full] [--all]
```

1. When celis asks for the `celis:past` agent, run it as celis says, even for a simple ask. With no such
   agent, do what the `past.md` the session start names says, yourself. Keep what it brings. When the ask conflicts with a rule or a decision, the ask wins: replace it with
   `decide --replaces <id>`.
2. Then decide whether the ask needs a trail. An ask that changes a file needs one, however small: a
   one-line edit too. So does an ask that works toward a result someone can check. Only an ask that
   changes no file needs none: a question, an explanation, a review or an investigation. Answer it,
   and run no command below.
3. Add the ask with `step add`. It goes under the session's trail, else it starts a trail. You decide
   whether the ask goes on with an active trail the agent brought, or with a related trail of today that
   `celis tree` prints: if it does, add it `--under <the trail's id>`. A trail ends when nothing in it is open. An
   ask that leaves the session's trail takes `--new`. It prints the ask's id
   first, and the trail's id second when it starts one. Write each rule the ask states with
   `decide --scope trail:<the trail's id>`. Give the ask the project's own check.
4. Go from the global to the local, one step ahead. For a larger ask, add 2 to 5 steps `--under` it, one
   for each big shape of the work, with a title only. Take the shape that removes the most doubt, or that
   the others need, and add one step under it, one level more local, until a step is one small action:
   one edit, one command or one read. Only that step gets `--at` and a `--check` that proves it. Put
   `expect: <the result>` on the line after its title. Add the next step after you close this one.
5. For each step: `start`, and keep the decisions it prints. Do the work, then `done` with a proof a
   person can check: `--as-planned` when the result matches `expect`, else `--changed`. Add `--pitfall`
   when something cost time. Replace a wrong shape with `--replaces <id>`.
6. When a step grows past one small action, or you try the same thing twice: stop. Write what you
   learned with `decide`, add smaller steps under the step, and take the first.
7. Do what celis prints after a command. It names the next move.
8. Close the ask, never the trail. Write each rule you learned that later work must keep with
   `decide --scope trail:<the trail's id>`, and tell the person each rule you wrote.

A rule reads "When <what you see>: <what to do>, because <the reason>." When the person asks you to
remember something, write it as one rule with `decide`, and `--replaces <id>` when it replaces one. Read
the `steps.md` the session start names for scopes, past work to copy, subagents, or another session's
trail.
