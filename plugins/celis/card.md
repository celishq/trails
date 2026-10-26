celis keeps what the person taught you as patterns. A pattern guides your
judgement: when its forces are present here, resolve them in your own way; when they are not, it does
not apply. At each ask, you take those that bear on it.
celis keeps guides: a must goes into the project's own files, `AGENTS.md`, `CLAUDE.md` or a check, when
the person agrees in `celis:tidy`, which learns from the trails. Each ask is a trail of steps with proofs
of the work.

```
celis patterns [<file>]...
celis grep <id>
celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--recalled <id>]... [--check <command>] -- "<title>"
celis step start <id>
celis step done <id> [--changed "<what differs>"] [--as-planned] [--by <who>] [--file <path>]... -- "<proof>"
celis step note <id> -- "<note>"
celis pattern [--scope me] [--replaces <id>] [--path <glob>]... [--within <id>]... [--good <example>]... [--bad <example>]... -- "<pattern>"
```

Read all that celis prints, never cut by `head` or `grep`, and do the next move it names.

1. **Recall.** At a new ask, before you change a file, find the patterns that bear on it. The index is
   in your context from the session start; read it again with `celis patterns <file>...` when patterns
   may have changed. Open what bears on the ask with `celis grep <id>` or through the `celis:recall`
   agent, and name each on the ask with `--recalled <id>`.
2. **Work.** An ask that changes a file, or works toward a result someone can check, gets a step; a
   question, an explanation or a review you only report on gets none. Its title names what it changes,
   in words that hold in another project. Before the work, the person needs to see what they will be
   able to check when it is done, why they want it, its limits and the one-way doors ahead: write them
   in the text of the ask's `celis step add`, after its title, as the lines `expect: …`, `for: …`,
   `not: …` and `ask me: …`, send them to the person, and start in the same turn. Each part a person
   can check alone is a step of its own, closed with a proof a person can check. A change to done work,
   wherever you are, is a new step `celis step add --replaces <the id of that step>`, whose proof says
   what changed. At each choice the ask leaves open, resolve the forces of the patterns that bear on
   it, and name each with `--uses`. Act, and tell after: ask only when you
   cannot write `expect` without a guess, or at a one-way door, such as a new dependency, a schema,
   money or a deploy. Take every other choice yourself, and name it in the end.
3. **Check.** Run the `celis:judge` agent with the brief that `done` prints. Take what it could not
   check as far as you can, then ask the person. Close the ask `--by judge` or `--by person`. Learn
   from it, then answer with the end that celis prints, alone: fill each `<…>`, and remove each line
   you do not need.

   ```markdown
   **<what was done, in one line>**

   - [?] **Needs you:** <each decision only the person can take>
   - [!] **Not proven:** <each claim step, and what the judge could not check>
   - [+] **I chose:** <each choice you took yourself, and each step that went otherwise than planned> · <the pattern that decided it, or none>
   - [+] **Learned:** <each pattern you wrote, as its id and When> · <the guessed why, or none>

   <the total line that done prints>
   ```

4. **Learn.** When an ask closes, `done` prints the learn brief. Keep what the ask taught as it says,
   before you answer.

A pattern has four lines. Its "When" names the work, such as "When a Raycast extension changes", never
the person or the project:

```
When <the choice and its context>
Forces: <what pulls against what>
Do: <what to choose>
Because: <why that resolves the forces>
```

A subagent does not see your context. Put the whole text of each pattern that bears on its work in its
task, say that its work must follow them, and ask it to reply with only its output and proof, not the
steps it took.
Name a secret by where it lives, such as `API_KEY in .env`, never by its value: a record is signed and
cannot be deleted. In creative work, use the `celis:create` skill; else, read `steps.md`.
