# celis — design

This page keeps the design decisions and why we took them. The contract is in `spec/`, and the code
follows the spec. A change of a decision changes this page first, then the spec, then the code. A plan
names the decisions it acts on.

## The core

celis keeps the knowledge that agents need to do work well, and the evidence for it.

- **Patterns are the core.** A pattern is a piece of knowledge: in a context, choose this, because of
  that. It activates in each trail whose context matches it.
- **A pattern guides.** As in Christopher Alexander's pattern language, it names the forces of a context
  and a way to resolve them. When its forces are present, the agent resolves them in its own way. When
  they are not present, the pattern does not apply. A pattern is never "broken". Pairs show taste: good
  and bad examples, made from the person's picks and corrections.
- **Trails are the evidence.** A trail is one ask, with its steps and their proofs. It gives observability,
  and it is the feedback loop that shows whether a pattern is still useful: its examples, its uses and
  tidy.

The store, the query, the card's method, the log, tidy and the tree are the core, because no model can
know them by itself. Hooks, ranks and caps are a scaffold, which a smarter model can outgrow.

```
  knowledge    pattern
               When · Forces · Do · Because
               good / bad examples, path globs
               scope: project | me
               within ──► a larger pattern
                  ▲                 │
   examples, uses, misled           │ activates
                  │                 ▼
  evidence     trail ──► step ──► done: proof, changed, by ──► note
```

## Decisions

1. **A pattern holds in a project or for me, never in one trail.** The `trail:<id>` scope goes. What
   holds for one trail only is a note on its step. A reader reads an old trail pattern as a note of
   that trail.
2. **One record: the pattern.** Its label is `pattern` in `celis.pattern`. A writer writes this label, and
   a reader still reads `decision` in `celis.decision`. A reader reads an old `binds` record as a plain
   pattern. Only the `grep` view marks it `old-rule`, so that tidy can find it. `pattern --binds` is
   accepted and ignored, so an older plugin still works with a newer `celis`. A pattern has no cell: a
   reader ignores an old `cell` tag, and `pattern --cell` is accepted and ignored in the same way.
3. **The agent loads a pattern at the choice it bears on.**
   - `celis patterns` gives the index: each live pattern in scope as its id and its When. `celis grep
     <id>` gives the whole pattern. celis has no vector, no word match and no rank. The session start
     gives the index, and gives it again after each compaction, so the agent has it before the first
     ask.
   - When a choice of the work matches a When, the agent reads that pattern whole before it chooses,
     and names it with `--uses` on the step of that choice. There is no step at an ask that reads the
     patterns, and no agent for it. The uses are the evidence. `step add --recalled` is accepted and
     ignored, and a reader ignores the `recalled` tag that older writers wrote on an ask.
   - `patterns <id>` writes a recall on the session's newest open step, and nothing when the session
     has no open step. A recall, then a use, then a step that no fix revises, is the funnel that shows
     whether the index finds the right pattern.
   - The agent marks a pattern that misled the work: `--misled` on the step that fixes done work, or on
     the `done` of a step where the pattern did not fit. A revision of a step that used a pattern still
     counts as misled, because the agent does not always mark it.
   - Past trails serve the learn and tidy. Nothing follows a past step: a step's `uses` names only a
     pattern, and a reader ignores a `uses` that names a step, which older writers wrote.
4. **What goes in matters most.** The agent decides when there is a lesson: a correction, a pick, a
   reaction to a result, a wish such as "from now on", or a fix of done work. It then runs `celis extract
   <id>`, which prints the learn brief of an ask or a step, open or closed: the ask's `changed` lines,
   its trail's notes, the person's prompts since the ask began, and the patterns in play, each one a
   step of the ask used, so that a lesson can replace one that misled. `done` of an ask, or of a step
   that fixes done work, because a fix is a correction, prints the same brief, and `--json` gives its
   parts to an integration. The `celis:extraction` agent writes the patterns from it, in its own
   context. The brief is the one place that says what to keep. A pattern is written only for a choice that will come back in other work, and never records what
   the ask did. A lesson replaces a pattern that misled the work instead of adding a near copy, and
   writes nothing when the ask taught nothing.
5. **The agent writes each record with a celis call.** A hook may give the agent context, such as the
   patterns that fit, but it never captures the agent's tools to write records.
6. **Patterns lead.** The card, the help and the README start with patterns, and trails come after.
7. **Patterns and trails are separate crates.** `celis-store` holds the store, the account, projects and
   ids. `celis-patterns` holds patterns, and `celis-trails` holds trails, steps and notes. Trails depend
   on patterns, and patterns never depend on trails. The repo keeps its name for now, and becomes
   `celis` later.
8. **`within` is the one link between patterns.** It goes from a smaller pattern to the larger one that
   it completes, as in Alexander's pattern language. A pattern can complete several larger ones. There
   are no other link types: `replaces` takes a pattern's place, and tidy settles conflicts.
9. **A must leaves celis.** celis keeps guides. A choice that must always hold goes into the project's
   own files: `AGENTS.md`, `CLAUDE.md`, or a check such as a test, a lint or CI. Tidy learns from the
   trails. From the uses, the misled and the replaces of each pattern, it offers to promote it, or to
   narrow or widen its When, and it draws new patterns from the choices of past trails. It writes a file
   only after the person agrees, and then drops the pattern with the reason "promoted to <file>".
10. **A step is open, done or dropped.** A start gives an open step a start time, and is not a state. A
    done trail that a drop names is dropped. An open step of an ended trail stays open, and the app
    shows it as stale, because a state by time is the app's.
11. **A trail is one ask.** The ask is a step with no `under`, and the steps of its work go under it. A
    session does not group its asks into a trail, so the agent never chooses between a trail and a new
    one: a new ask is a new trail, and the agent continues an open ask with `--under`. The prompts and
    notes of an ask are on it. A reader reads an older trail, which held the asks of a session, as one
    ask with those asks as its steps. `step add --new` is accepted and ignored. The migrate skill closes
    each older trail after the person agrees.
12. **No judge.** celis keeps patterns, and to check work is the orchestration's. A step proves its work
    with its own check or its proof, and `done` gives the patterns of the changed files to check the
    change against. `done --by person` says that the person confirms the result. A reader reads an old
    `by judge` as a proof by a reader that did not do the work. `done --by judge` is accepted and writes
    no `by`, so an older plugin still works with a newer `celis`.
13. **The outcomes are evidence, and celis lists what needs tidy.** A pattern misled a step that used
    it when a later step fixed that work. `celis patterns --tidy` lists each live pattern that misled
    twice or more, that has a near copy, that no step used in 60 days, or that was replaced three times
    or more, with the reason. Tidy narrows a pattern that misled, and drops or widens one unused.
    `celis stats` shows the use of each live pattern: its uses, misled, replaces, last use, examples and
    tidy reason. Its total gives the size of the index, the patterns written and replaced in the last 7
    days, and the gaps: the fixes of done work in that time whose ask used no pattern. Its lifecycle
    gives, by day, week and month, the patterns written, replaced, dropped, promoted, recalled, used and
    misled. Its value gives, for the last 28 days, what shows that celis pays: the asks that used a
    pattern, the corrections and the repeat ones among them, first-pass success with and without a
    pattern, the recall funnel, the reach of the patterns to other projects and agents, and the
    promotions. A repeat correction failed when the fixed work used a pattern, and missed when it used
    none but its lesson replaced a pattern that was live before the work. A step names its agent when the
    variable that gives its session names one. The session start offers tidy when the list
    is not empty, and not within 7 days of a tidy, so that a person who kept a pattern is not asked each
    session.
14. **celis refuses a secret.** A record is signed and cannot be deleted, so a secret in it leaks for
    good. A writer refuses a record, and a kept file, that holds a secret, such as an API key, a token, a
    private key or a password, with the error `secret`, and the agent writes it again with the secret
    named by where it lives, such as `API_KEY in .env`. The prompt hook keeps no prompt that holds a
    secret. A guide in the card alone did not hold: agents gave `.env` as a proof file.

15. **An old name is deprecated, and still works.** celis accepts each name that an older plugin gives,
    such as `lesson`, `--binds`, `--cell`, `--new`, `--recalled` and `--by judge`, does what its row in
    the spec says, and prints one line on stderr that names it as deprecated. So an older plugin works
    with a newer `celis`, and its agent learns the new name. A reader reads each old record as before,
    because a record cannot be deleted.

## What we do not do

- No rank or grade of work by time.
- No model, no index and no vector search in celis. The agent's own model reads the patterns.
- No sharing across a team or an org yet.

## Open questions

- Which step duties can move from the agent to hooks, while decision 5 holds?
- Which `--json` keys change with the label?
