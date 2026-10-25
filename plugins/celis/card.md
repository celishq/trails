celis keeps each ask as a trail of steps with proofs, and what the person taught you: rules, which
bind, and patterns, good and bad examples, which guide. A rule wins over a pattern.

```
celis search [<file>]... -- "<ask>"
celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--check <command>] [--new] -- "<title>"
celis step start <id>
celis step done <id> [--changed "<what differs>"] [--as-planned] [--by <who>] [--file <path>]... -- "<proof>"
celis decide [--scope trail:<id> | me] [--replaces <id>] [--path <glob>]... [--good <example>]... [--bad <example>]... -- "<rule>"
```

Read all that celis prints, never cut by `head` or `grep`, and do the next move it names.

1. **Search.** At a new ask, before you change a file, run
   `celis search [<each file it will touch>]... -- "<the ask>"`.
2. **Work.** An ask that changes a file, or works toward a result someone can check, gets a step; a
   question, an explanation or a review you only report on gets none. Its title is a verb and what it
   acts on, in words that hold in another project. Under it, write `expect: <what a person can check
   when it is done>`, `for: <why the person wants it>`, `not: <its limits>` and `ask me: <the one-way
   doors you see ahead>`. Send these lines to the person, give the ask the project's own check, and
   start in the same turn. Each step gets `start`, then `done` with a proof a person can check, such
   as a command and its result.
   Act, and tell after: ask only when you cannot write `expect` without a guess, or at a one-way door,
   such as a new dependency, a schema, money or a deploy. Take every other choice yourself, name it in
   the end, and write nothing else until then.
3. **Check.** Run the `celis:judge` agent with the brief that `done` prints. Take what it could not
   check as far as you can, then ask the person. Close the ask `--by judge` or `--by person`, never the
   trail, and answer with the end that celis prints, alone: fill each `<…>`, remove each line you do
   not need, and add each rule and pattern you wrote.

   ```markdown
   **<what was done, in one line>**

   - [?] **Needs you:** <each decision only the person can take>
   - [!] **Not proven:** <each claim step, and what the judge could not check>
   - [+] **I chose:** <each choice you took yourself, and each step that went otherwise than planned> · <the pattern that decided it, or none>

   <the total line that done prints>
   ```

4. **Learn.** When the person corrects you or reacts to a result, or something surprises you, write
   it with `decide` before you go on. A correction that says "always" or "never", or names a
   convention, is a rule with no examples. A reaction to a result, such as "this one" or "too busy",
   is a pattern, with `--good` and `--bad` (`steps.md`, "Patterns"). Give it the narrowest `--path`,
   and replace a wrong rule with `--replaces <id>`.

A rule reads "When <what you see>: <what to do>, because <the reason>." Its "When" names the work,
such as "When a Raycast extension changes", never the person or the project.

A subagent gets the rules and patterns of its work in its task, and replies with its output and proof.
Name a secret by where it lives, such as `API_KEY in .env`, never by its value: a record is signed and
cannot be deleted. In creative work, use the `celis:create` skill; for other cases, read `steps.md`.
