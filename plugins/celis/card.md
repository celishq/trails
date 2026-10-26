celis keeps what the person taught you as patterns. A pattern guides your
judgement: when its forces are present here, resolve them in your own way; when they are not, it does
not apply. Their index, one `<id> <scope> <When>` line each, is in your context from the session start.
When a choice of the work matches a When, read that pattern whole with `celis grep <id>` before you
choose, and name it with `--uses <id>` on the step of that choice.
celis keeps guides: a must goes into the project's own files, `AGENTS.md`, `CLAUDE.md` or a check, when
the person agrees in `celis:tidy`, which learns from the trails. Each ask is a trail of steps with proofs
of the work.

```
celis patterns [<file>]...
celis grep <id>
celis lesson <id>
celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--check <command>] -- "<title>"
celis step start <id>
celis step done <id> [--changed "<what differs>"] [--as-planned] [--by person] [--file <path>]... -- "<proof>"
celis step note <id> -- "<note>"
celis pattern [--scope me] [--replaces <id>] [--path <glob>]... [--within <id>]... [--good <example>]... [--bad <example>]... -- "<pattern>"
```

Read all that celis prints, never cut by `head` or `grep`, and do the next move it names.

1. **Work.** An ask that changes a file, or works toward a result someone can check, gets a step; a
   question, an explanation or a review you only report on gets none. Its title names what it changes,
   in words that hold in another project. Before the work, the person needs to see what they will be
   able to check when it is done, why they want it, its limits and the one-way doors ahead: write them
   in the text of the ask's `celis step add`, after its title, as the lines `expect: …`, `for: …`,
   `not: …` and `ask me: …`, send them to the person, and start in the same turn. Each part a person
   can check alone is a step of its own, closed with a proof a person can check. A change to done work,
   wherever you are, is a new step `celis step add --replaces <the id of that step>`, whose proof says
   what changed. Act, and tell after: ask only when you cannot write `expect` without a guess, or at a
   one-way door, such as a new dependency, a schema, money or a deploy. Take every other choice
   yourself, and name it in the end.
2. **Close.** When the work of the ask is done, close it with its proof. When only the person can check
   it, ask them, and close it `--by person` with their answer. Learn from it, then answer with the end
   that celis prints, alone: fill each `<…>`, and remove each line you do not need.

   ```markdown
   **<what was done, in one line>**

   - [?] **Needs you:** <each decision only the person can take>
   - [!] **Not proven:** <each claim step, and what no check proves>
   - [+] **I chose:** <each choice you took yourself, and each step that went otherwise than planned> · <the pattern that decided it, or none>
   - [+] **Learned:** <each pattern you wrote, as its id and When> · <the guessed why, or none>

   <the total line that done prints>
   ```

3. **Learn.** A lesson is a correction, a pick, a reaction to a result, a wish such as "from now on",
   or a fix of done work. When you see one, run `celis lesson <the ask's id>`; `done` of an ask or of a
   fix prints the same brief. Give the brief to the `celis:lesson` agent, and name what it wrote on the
   Learned line of the end.

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
