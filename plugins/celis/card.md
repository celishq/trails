celis keeps what the person taught you as patterns, in cells by topic. A pattern guides your
judgement: when its forces are present here, resolve them in your own way; when they are not, it does
not apply. Search gives those that fit an ask. Write one only for a choice that will come back. Each ask is a trail of
steps with proofs of the work.

```
celis search [<file>]... -- "<ask>"
celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--check <command>] [--new] -- "<title>"
celis step start <id>
celis step done <id> [--changed "<what differs>"] [--as-planned] [--by <who>] [--file <path>]... -- "<proof>"
celis step note <id> -- "<note>"
celis pattern [--scope me] [--replaces <id>] [--path <glob>]... [--within <id>]... [--cell <topic>]... [--good <example>]... [--bad <example>]... -- "<pattern>"
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
   as a command and its result. A step with more than one part a person can check alone holds a step
   for each, with its own `expect`; when its last part is done, it gets the judge too.
   At each choice the ask leaves open, take the patterns that search gave for it and the `next` ones
   under them whose forces are present, resolve those forces, and add each pattern that guided the
   choice to the step with `--uses`.
   Act, and tell after: ask only when you cannot write `expect` without a guess, or at a one-way door,
   such as a new dependency, a schema, money or a deploy. Take every other choice yourself, name it in
   the end, and write nothing else until then.
3. **Check.** Run the `celis:judge` agent with the brief that `done` prints. Take what it could not
   check as far as you can, then ask the person. Close the ask `--by judge` or `--by person`, never the
   trail. Learn from it, then answer with the end that celis prints, alone: fill each `<…>`, and remove
   each line you do not need.

   ```markdown
   **<what was done, in one line>**

   - [?] **Needs you:** <each decision only the person can take>
   - [!] **Not proven:** <each claim step, and what the judge could not check>
   - [+] **I chose:** <each choice you took yourself, and each step that went otherwise than planned> · <the pattern that decided it, or none>
   - [+] **Learned:** <each pattern you wrote, as its id and When>

   <the total line that done prints>
   ```

4. **Learn.** When an ask closes, `done` gives what it taught: its `changed` lines, its trail's notes
   and the person's prompts. Write a pattern for each choice that will come back, with `--replaces <id>`
   for a pattern that misled you, and nothing when the ask taught nothing. A correction that says
   "always" or "never", or names a convention, is a pattern whose forces say when it applies. A reaction to a result, such as "this
   one" or "too busy", is a pattern with `--good` and `--bad` (`steps.md`, "Patterns"). Give it the narrowest `--path`, name
   the larger pattern it completes with `--within <id>`, and replace a wrong pattern or a near copy
   with `--replaces <id>`. Give it one to three cells with `--cell <topic>`, and reuse a cell that
   search shows before you make a new one. What holds only for this trail is a note:
   `step note <the ask's id>`.

A pattern has four lines. Its "When" names the work, such as "When a Raycast extension changes", never
the person or the project:

```
When <the choice and its context>
Forces: <what pulls against what>
Do: <what to choose>
Because: <why that resolves the forces>
```

A subagent gets the patterns of its work in its task, and replies with its output and proof.
Name a secret by where it lives, such as `API_KEY in .env`, never by its value: a record is signed and
cannot be deleted. In creative work, use the `celis:create` skill; else, read `steps.md`.
