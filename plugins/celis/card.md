celis keeps each ask as a trail: its steps, their proofs, and the rules that later work must keep.

```
celis rules [<file>]... [--diff]
celis grep <grep option>... "<pattern>"
celis decide [--scope at:<place> | trail:<id> | me] [--replaces <id>] [--path <glob>]... [--situation] [--promoted] [--project <id>] -- "<rule>"
celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--at <place>] [--check <command>] [--new] [--session <id>] [--project <id>] -- "<title>"
celis step start <id>
celis step done <id> [--changed "<what differs>"] [--as-planned] [--pitfall "<rule>"] -- "<proof>"
celis step drop <id> -- "<why>"
celis step edit <id> [--at <place>]... [--check <command>] [-- "<new title>"]
celis name <trail id> <name>
celis tree [<id>] [--graph] [--full] [--all] [--demo]
```

1. At a new ask, run the `celis:past` agent when celis asks for it, even for a simple ask: it brings
   the rules of the ask and past steps like it. Run it in the foreground, never in the background, and
   wait for its answer before any other tool. To find more rules, run `celis rules` for the words the
   rules use, and search them with `celis grep -i -E " rule [0-9a-f]{8} live .*(<word>|<word>)"`. Keep
   each rule in your own context. When the ask conflicts with a rule, the ask wins: replace the rule.
2. An ask that changes a file, however small, or that works toward a result someone can check, gets a
   step: review feedback to fix too. A question, an explanation, or a review or investigation you only
   report on gets none: answer it. Add the ask with `step add` where the prompt hook says. With no hint,
   add it under the session's trail, else with `--new`. An ask that corrects done work takes
   `--replaces <its id>`. Give the ask the project's own check, and `--uses <id>` for each rule, past
   step or named trail it follows. `--uses` takes a trail's name.
3. A step's title names what the person wants done: a verb and what it acts on, in words that hold in
   another project. On the line after the title, write `expect: <what a person can check when it is
   done>`. Split an ask by its jobs: a job is a part of the ask that a person can check alone, such as
   an item of its list or a module. An ask with one job, such as a function and its test, is one step. Add the jobs as the work
   goes: the first job when you start, the next job after you close one, and steps under a job only when
   you take it and it holds more than one job. Only the step you take next gets `--at` and a `--check`
   that proves it.
4. For each step: `start`, and keep the rules it prints. Run `celis rules <files>` before each change,
   and before `done`, check your change against `celis rules --diff`. Read all
   that celis prints: never cut it with `head` or `grep`. Then `done` with a proof a person can check,
   such as a command and its result: `--as-planned` when the result matches `expect`, else
   `--changed`. A wrong step, or a fix to done work, takes `--replaces <id>`. When a step grows past one
   action, or you try the same thing twice, stop and split it. When you give work to a subagent, put in
   its task each rule of that work, as celis printed it. Name a secret by where it lives, such as
   `API_KEY in .env`, never by its value: a record is signed and cannot be deleted.
5. When something surprises you, or the person corrects you, write the rule that would have saved you
   with `decide`, before you go on. Give it `--path` for the files it acts on, as narrow as the rule
   allows, and `--situation` only when no file shows it ahead. A
   correction replaces the wrong rule with `--replaces <id>`: never keep two rules for one point.
6. Do what celis prints after each command. It names the next move.
7. Close the ask, never the trail. Tell the person about the ask by the steps right under it, and each
   rule you wrote.

A rule reads "When <what you see>: <what to do>, because <the reason>." The "When" names the work the rule
acts on, such as "When a Raycast extension changes", never the review, the person or the project it came
from, so it matches the next task like it. The trigger says when the rule comes; the text stays general.
Its scope is `trail:<id>` when it holds for this trail, `project` (the default) for the repo, and `me`
when it holds for that kind of work in any project: a rule about a framework, a platform or a review's
convention. When the person asks you to remember something, or states how the work must be done, write
each point as one rule before you do the work.

For more: `celis tree` shows the work. Read the `steps.md` the session start names for moved files, past
work, named trails, subagents, links, and when to stop.
