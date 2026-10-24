celis keeps each ask as a trail: its steps, their proofs, and the rules that later work must keep.

```
celis rules [<file>]... [--diff]
celis decide [--scope at:<place> | trail:<id> | me] [--replaces <id>] [--path <glob>]... [--situation] [--good <id>] [--bad <id>] -- "<rule>"
celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--at <place>] [--check <command>] [--new] -- "<title>"
celis step start <id>
celis step done <id> [--changed "<what differs>"] [--as-planned] [--by <who>] [--file <path>]... [--pitfall "<rule>"] -- "<proof>"
celis step drop <id> -- "<why>"
celis step edit <id> [--at <place>]... [--check <command>] [-- "<new title>"]
celis tree [<id>]...
```

1. An ask that changes a file, however small, or that works toward a result someone can check, gets a
   step; a question, an explanation, or a review you only report on gets none: answer it. Its title
   names what the person wants done: a verb and what it acts on, in words that hold in another
   project. On the lines after it, write `expect: <what a person can check when it is done>`,
   `for: <why the person wants it>`, `not: <its limits>` and `ask me: <the one-way doors you see
   ahead>`. Give it the project's own check. Split it by its jobs: a job is a part a person can check
   alone, such as an item of its list or a module; an ask with one job, such as a function and its test,
   is one step. A feature with more than one part, such as a tool with several commands, or creative
   work, is never one step: its jobs are layers. The outer layer makes the whole result at low detail,
   such as every command bare but running end to end, a greybox or an outline, and each next layer adds
   detail to all of it. Only the step you take next gets `--at` and a `--check` that proves it.
2. Keep in your own context each rule that the past agent brings. For each step: `start`, and keep the
   rules it prints. Run `celis rules <files>` before each change,
   and before `done`, check your change against `celis rules --diff`. Read all that celis prints: never
   cut it with `head` or `grep`. Then `done` with a proof a person can check, such as a command and its
   result: `--as-planned` when the result matches `expect`, else `--changed`. A wrong step, or a fix to
   done work, takes `--replaces <id>`. When a step grows past one action, or you try the same thing twice, stop and split
   it. A subagent gets each rule of its work in its task, and replies with its output and its proof
   only. Name a secret by where it lives, such as `API_KEY in .env`, never by its value: a record is
   signed and cannot be deleted.
3. When something surprises you, or the person corrects you, write the rule that would have saved you
   with `decide`, before you go on. Give it `--path` for the files it acts on, as narrow as the rule
   allows, and `--situation` only when no file shows it ahead. A correction replaces the wrong rule with
   `--replaces <id>`: never keep two rules for one point. When the person corrects done work, keep the
   fix as a taste block (`steps.md`, "Taste").
4. Act, and tell after. Ask before the work only when you cannot write the `expect` without a guess,
   and during the work only at a one-way door: a choice that is hard to undo, such as a new dependency,
   a schema, money or a deploy. Take every other choice yourself, and name it in the end. After the
   intent, write nothing but such a question until the end: the person reads the end, not the work.
5. Before you close the ask, run the `celis:judge` agent with its intent, where the result is, and its
   checks; with no subagents, do what `agents/judge.md` beside `steps.md` says. What the judge could not
   check, take as far as you can, such as a run or a screenshot, then ask the person. Close the ask
   `--by judge` with its verdict, or `--by person` with the person's answer, and never close the trail.
   Then answer with this block alone, with no prose before it, each line only when it is not empty, and
   add each rule you wrote:

   ```
   Done:       <what was done, in one line>
   Turn:       <each changed and edited line that done prints, and why>
   Took:       <the choices you took yourself, such as the variant you chose>
   Not proven: <each claim line that done prints, and what the judge could not check>
   Outside:    <the files git diff --stat shows changed outside the places line>
   Needs you:  <the decisions>
   <the total line that done prints>
   ```

   When the person changes the intent, `step edit` the ask with the new lines.
6. Do what celis prints after each command: it names the next move.

A rule reads "When <what you see>: <what to do>, because <the reason>." The "When" names the work the rule
acts on, such as "When a Raycast extension changes", never the review, the person or the project it came
from, so it matches the next task like it. Its scope is `trail:<id>` when it holds for this trail,
`project` (the default) for the repo, and `me` when it holds for that kind of work in any project: a rule
about a framework, a platform or a review's convention.

In creative work, such as an image, a sound, a level or a text, use the `celis:create` skill. For the
cases the card does not cover, read the `steps.md` the session start names.
