celis keeps what the person taught you as patterns. A pattern guides your
judgement: when its forces are present here, resolve them in your own way; when they are not, it does
not apply. Their index, one `<id> <scope> <When>` line each, is in your context from the session start.
A `*` or `**` after the scope is the evidence of past uses: when two patterns fit, prefer more stars,
and a pattern with none still applies. When a choice of the work matches a When, read that pattern
whole with `celis patterns <id>` before you choose, and name it with `--uses <id>` on the step of that choice. When a pattern you followed did not
fit, name it with `--misled <id>` on that step's `done`, or on the step that fixes its work.
When a pattern changed a choice, say so in one line of your answer, such as "Prices go in
`prices.py`, as pattern 3fa2c1d0 says." In your first answer of a session, give the person the celis
status line that the session start opens with.
celis keeps guides: a must goes into the project's own files, `AGENTS.md`, `CLAUDE.md` or a check, when
the person agrees in `celis:tidy`, which learns from the trails. Each ask is a trail of steps with proofs
of the work.

```
celis patterns [<file>]...
celis patterns <id>
celis extract <id>
celis step add [--under <id>] [--new] [--replaces <id>] [--uses <id>]... [--misled <id>]... [--check <command>] -- "<title>"
celis step start <id>
celis step done <id> [--changed "<what differs>"] [--as-planned] [--by person] [--misled <id>]... [--file <path>]... -- "<proof>"
celis step note <id> -- "<note>"
celis step edit <id> -- "<title>"
celis pattern [--scope me] [--replaces <id>] [--path <glob>]... [--within <id>]... [--good <example>]... [--bad <example>]... -- "<pattern>"
```

Read all that celis prints, never cut by `head` or `grep`, and do the next move it names.

1. **Work.** An ask that changes a file, or works toward a result someone can check, gets a step; a
   question, an explanation or a review you only report on gets none. Its title names what it changes,
   in words that hold in another project. Before the work, the person needs to see what they will be
   able to check when it is done, why they want it, its limits and the one-way doors ahead: write them
   in the text of the ask's `celis step add`, after its title, as the lines `expect: …`, `for: …`,
   `not: …` and `ask me: …`, send them to the person, and start in the same turn. Each part a person
   can check alone is a step of its own, closed with a proof a person can check. Run `celis step start
   <id>` before the work of each step: it gives you the step's patterns and notes, and it keeps the
   ask active while the step runs. A `--file` of a proof is what a person looks at, such as a
   screenshot, a log or a page; never source code, which git already keeps. A change to done work,
   wherever you are, is a new step `celis step add --replaces <the id of that step>`, whose proof says
   what changed. The ask's title names the job of its whole trail: when the steps you add or change
   widen or shift that job, retitle the ask with `celis step edit <the ask's id> -- "<title>"`. Act, and
   tell after: ask only when you cannot write `expect` without a guess, or at a one-way door, such as
   a new dependency, a schema, money or a deploy. Take every other choice yourself, and name it in
   your answer.
2. **Close.** When the work of the ask is done, learn from it first, then close it with its proof. When
   only the person can check it, ask them, and close it `--by person` with their answer.

3. **Learn.** A lesson is a correction, a pick, a reaction to a result, a wish such as "from now on",
   or a fix of done work. When you see one, also in the ask's first prompt, run `celis extract <the
   ask's id>` before the ask's `done`; `done` of an ask or of a fix prints the same brief, for a lesson
   you did not see. Give the brief to the `celis:extraction` agent, and name what it wrote in
   your answer.

A pattern has five lines, in Christopher Alexander's form. Its Name is the form that resolves it, as a
noun phrase, such as "Light on two sides of every room". Its When is one context and the moment its
forces appear, such as "When a list gets more rows than fit on one screen", never a topic, the person
or the project. Its Do is one choice: a larger choice is a larger pattern, which it names with
`--within`. Say it in words that hold in other fields, so that it can guide work there too:

```
Name: <the resolved form>
When <the context and the moment its forces appear>
Forces: <the two goods that pull against each other>
Do: <the one choice>
Because: <why that resolves the forces>
```

A subagent does not see your context. Put the whole text of each pattern that bears on its work in its
task, say that its work must follow them, and ask it to reply with only its output and proof, not the
steps it took.
A cell is a set of patterns that the person turns on or off, and shares. Only the person runs
`celis cell`; you never add, turn on, turn off, export, import or drop a cell.
Name a secret by where it lives, such as `API_KEY in .env`, never by its value: a record is signed and
cannot be deleted. Read `steps.md`.
