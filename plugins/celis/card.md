celis keeps each ask as a trail of steps with proofs, and what the person taught you: rules, which the
work must keep, and patterns, the person's good and bad examples, which guide your choices.

```
celis search [<file>]... -- "<ask>"
celis decide [--scope trail:<id> | me] [--replaces <id>] [--path <glob>]... [--situation] [--good <example>]... [--bad <example>]... -- "<rule>"
celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--at <place>] [--check <command>] [--new] -- "<title>"
celis step start <id>
celis step done <id> [--changed "<what differs>"] [--as-planned] [--by <who>] [--file <path>]... [--pitfall "<rule>"] -- "<proof>"
celis step drop <id> -- "<why>"
celis step edit <id> [--at <place>]... [--check <command>] [-- "<new title>"]
celis tree [<id>]...
```

Each ask is one loop of four moves: search, work, check, learn.

1. **Search.** At each new ask, run `celis search [<each file it will touch>]... -- "<the ask>"`
   before you change a file. Keep each rule it prints: a rule binds. Keep each pattern: at each choice
   the ask leaves open, choose like its good examples and unlike its bad ones; a pattern guides, and a
   rule wins over it. Open the files of the examples that bear on your choice. It also brings 1 or 2
   done asks like this one: take each of their steps that fits as yours, with `--uses <its id>`
   (`steps.md`, "Past work").
2. **Work.** An ask that changes a file, however small, or that works toward a result someone can
   check, gets a step; a question, an explanation, or a review you only report on gets none: answer
   it. Its title names what the person wants done: a verb and what it acts on, in words that hold in
   another project. On the lines after it, write `expect: <what a person can check when it is done>`,
   `for: <why the person wants it>`, `not: <its limits>` and `ask me: <the one-way doors you see
   ahead>`. Write these lines to the person in one message, and start in the same turn. Give the ask
   the project's own check. When the ask is long, split it by its jobs: a job is a part a person can
   check alone, such as an item of its list or a module; an ask with one job, such as a function and
   its test, is one step. A step says what it does and why; its `--at` and `--check` are optional, and
   only the ask needs a check. Add only the step you take next. For each step:
   `start`, and keep the rules it prints. Read all that
   celis prints: never cut it with `head` or `grep`. Then `done` with a proof a person can check, such
   as a command and its result: `--as-planned` when the result matches `expect`, else `--changed`. A
   wrong step, or a fix to done work, takes `--replaces <id>`. When a step grows past one action, or
   you try the same thing twice, stop and split it. A subagent gets each rule and pattern of its work
   in its task, and replies with its output and its proof only. Name a secret by where it lives, such
   as `API_KEY in .env`, never by its value: a record is signed and cannot be deleted.
   Act, and tell after. Ask before the work only when you cannot write the `expect` without a guess,
   and during the work only at a one-way door: a choice that is hard to undo, such as a new
   dependency, a schema, money or a deploy. Take every other choice yourself, and name it in the end.
   After the intent, write nothing but such a question until the end: the person reads the end, not
   the work.
3. **Check.** `done` prints the rules of the files you changed: check your change against each.
   Before you close the ask, run the `celis:judge` agent with its intent, the rules and patterns that
   search gave, where the result is, and its checks; with no subagents, do what `agents/judge.md` beside `steps.md` says.
   What the judge could not check, take as far as you can, such as a run or a screenshot, then ask the
   person. Close the ask `--by judge` with its verdict, or `--by person` with the person's answer, and
   never close the trail. Then answer with the end that celis prints, alone, with no prose before it.
   Fill each `<…>`, and remove each line you do not need, so that an ask with all its steps checked
   ends in two lines. Add each rule and pattern you wrote. Each "I chose" names the pattern behind
   the choice, or says that none did:

   ```markdown
   **<what was done, in one line>**

   - [?] **Needs you:** <each decision only the person can take>
   - [!] **Not proven:** <each claim step, and what the judge could not check>
   - [+] **I chose:** <each choice you took yourself, and each step that went otherwise than planned> · <the pattern that decided it, or none>

   <the total line that done prints>
   ```

4. **Learn.** When something surprises you, or the person corrects you, write what would have saved
   you with `decide`, before you go on. A correction that says always or never, or names a convention,
   is a rule. A reaction to a result, such as "this one", "too busy" or a fix of done work, is a
   pattern: give it `--good` for what the person liked and `--bad` for what they did not, each a step,
   a file or a URL (`steps.md`, "Patterns"). Give it `--path` for the files it acts on, as narrow as
   it allows, and `--situation` only when no file shows it ahead. A correction replaces the wrong rule
   or pattern with `--replaces <id>`: never keep two for one point. When the person changes the
   intent, `step edit` the ask with the new lines.

Do what celis prints after each command: it names the next move.

A rule or a pattern reads "When <what you see>: <what to do>, because <the reason>." The "When" names
the work it acts on, such as "When a Raycast extension changes", never the review, the person or the
project it came from, so it matches the next task like it. Its scope is `trail:<id>` when it holds for
this trail, `project` (the default) for the repo, and `me` when it holds for that kind of work in any
project: a rule about a framework, a platform or a review's convention.

In creative work, such as an image, a sound, a level or a text, use the `celis:create` skill. For the
cases the card does not cover, read the `steps.md` the session start names.
