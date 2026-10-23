celis keeps each ask as a trail: its steps, their proofs, and the rules that later work must keep.

```
celis rules [<file>]... [--diff]
celis grep <grep option>... "<pattern>"
celis decide [--scope at:<place> | trail:<id> | me] [--replaces <id>] [--path <glob>]... [--situation] [--promoted] [--good <id>] [--bad <id>] [--project <id>] -- "<rule>"
celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--at <place>] [--check <command>] [--new] [--session <id>] [--project <id>] -- "<title>"
celis step start <id>
celis step done <id> [--changed "<what differs>"] [--as-planned] [--by <who>] [--file <path>]... [--pitfall "<rule>"] -- "<proof>"
celis step drop <id> -- "<why>"
celis step edit <id> [--at <place>]... [--check <command>] [-- "<new title>"]
celis name [<trail id> <name>] [--limit <n>]
celis tree [<id>]... [--graph] [--full] [--all] [--markdown] [--demo]
```

1. At a new ask, run the `celis:past` agent when celis asks for it, even for a simple ask: it brings
   the rules of the ask and past steps like it. Run it in the foreground, never in the background, and
   wait for its answer before any other tool. To find more rules, run `celis rules` for the words the
   rules use, and search them with `celis grep -i -E " rule [0-9a-f]{8} live .*(<word>|<word>)"`. Keep
   each rule in your own context. When the ask conflicts with a rule, the ask wins: replace the rule.
   In creative work, rules guide and do not bind (`steps.md`, "Creative work").
2. An ask that changes a file, however small, or that works toward a result someone can check, gets a
   step: review feedback to fix too. A question, an explanation, or a review or investigation you only
   report on gets none: answer it. Add the ask with `step add` where the prompt hook says. With no hint,
   add it under the session's trail, else with `--new`. An ask that corrects done work takes
   `--replaces <its id>`. Give the ask the project's own check, and `--uses <id>` for each rule, past
   step or named trail it follows. `--uses` takes a trail's name.
3. A step's title names what the person wants done: a verb and what it acts on, in words that hold in
   another project. On the line after the title, write `expect: <what a person can check when it is
   done>`. An ask larger than a small, clear fix also gets its intent on the lines after it:
   `for: <why the person wants it>`, `not: <its limits>` and `ask me: <the one-way doors you see
   ahead>`. Show the person these lines before the work: they are what you both agree done means.
   Split an ask by its jobs: a job is a part of the ask that a person can check alone, such as
   an item of its list or a module. An ask with one job, such as a function and its test, is one step. Add the jobs as the work
   goes: the first job when you start, the next job after you close one, and steps under a job only when
   you take it and it holds more than one job. Only the step you take next gets `--at` and a `--check`
   that proves it.

   A feature or creative work, such as an image, a level or a text, splits by layers, each one job:
   the outer layer makes the whole result at low detail first, such as a skeleton that runs end to end,
   a greybox or an outline, and each next layer adds detail to all of it. In creative work, read
   "Creative work" in `steps.md` before the outer layer.
4. For each step: `start`, and keep the rules it prints. Run `celis rules <files>` before each change,
   and before `done`, check your change against `celis rules --diff`. Read all
   that celis prints: never cut it with `head` or `grep`. Then `done` with a proof a person can check,
   such as a command and its result: `--as-planned` when the result matches `expect`, else
   `--changed`. A wrong step, or a fix to done work, takes `--replaces <id>`. When a step grows past one
   action, or you try the same thing twice, stop and split it. When you give work to a subagent, put in
   its task each rule of that work, as celis printed it, and ask it to reply with its output and its
   proof only. Name a secret by where it lives, such as
   `API_KEY in .env`, never by its value: a record is signed and cannot be deleted.
5. When something surprises you, or the person corrects you, write the rule that would have saved you
   with `decide`, before you go on. Give it `--path` for the files it acts on, as narrow as the rule
   allows, and `--situation` only when no file shows it ahead. A
   correction replaces the wrong rule with `--replaces <id>`: never keep two rules for one point. When
   the person corrects done work, put the hunk it replaced and the fix in the fix's proof, and give its
   rule `--good <the fix>` (`steps.md`, "Taste").
6. Do what celis prints after each command. It names the next move.
7. Act, and tell after. When you can write the `expect` of an ask without a guess, show its intent and
   start. Ask before the work only when you cannot, and during the work only at a one-way door: a
   choice that is hard to undo, such as a new dependency, a schema, money or a deploy. Take every other
   choice yourself, and name it in the end. Between steps, write nothing but such a question: the
   person reads the end, not the work.
8. Close the ask, never the trail. Before you close it, run the `celis:judge` agent with the ask's
   intent, where the result is, and its checks; with no subagents, do what `agents/judge.md` beside
   `steps.md` says. What the judge could not check, take as far as you can, such as a run or a
   screenshot, then ask the person. Close the ask `--by judge` with its verdict, or `--by person` with
   the person's answer. Then tell the person the end, each line only when it is not empty:

   ```
   Done:       <what was done, in one line>
   Turn:       <each changed and edited line that done prints, and why>
   Took:       <the choices you took yourself, such as the variant you chose>
   Not proven: <each claim line that done prints, and what the judge could not check>
   Outside:    <the files git diff --stat shows changed outside the places line>
   Needs you:  <the decisions>
   <the total line that done prints>
   ```

   Add each rule you wrote. When the person changes the intent, `step edit` the ask with the new
   lines.

A rule reads "When <what you see>: <what to do>, because <the reason>." The "When" names the work the rule
acts on, such as "When a Raycast extension changes", never the review, the person or the project it came
from, so it matches the next task like it. The trigger says when the rule comes; the text stays general.
Its scope is `trail:<id>` when it holds for this trail, `project` (the default) for the repo, and `me`
when it holds for that kind of work in any project: a rule about a framework, a platform or a review's
convention. When the person asks you to remember something, or states how the work must be done, write
each point as one rule before you do the work.

For more: `celis tree` shows the work. Read the `steps.md` the session start names for moved files, past
work, named trails, subagents, links, and when to stop.
