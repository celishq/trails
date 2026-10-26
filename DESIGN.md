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
- **Trails are the evidence.** A trail records an ask, its steps and their proofs. It gives observability,
  and it is the feedback loop that shows whether a pattern is still useful: its examples, its uses, its
  recalls, the judge and tidy.

The store, the query, the card's method, the log, tidy and the tree are the core, because no model can
know them by itself. Hooks, ranks and caps are a scaffold, which a smarter model can outgrow.

```
  knowledge    pattern
               When · Forces · Do · Because
               good / bad examples, path globs
               scope: project | me
               within ──► a larger pattern
                  ▲                 │
   examples, uses, recalls          │ activates
                  │                 ▼
  evidence     trail ──► step ──► done: proof, changed, judge ──► note
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
3. **Search finds the best patterns.**
   - Patterns are ranked by how well they fit: by path, then by words. The output shows
     only a budget of them.
   - The past of like asks is a helper part, and it is short.
4. **What goes in matters more than search.** An agent writes a pattern only for a choice that will come
   back. It replaces a pattern instead of adding a near copy.
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

## What we do not do

- No rank or grade of work by time.
- No model, no index and no vector search in celis. The agent's own model reads the patterns.
- No sharing across a team or an org yet.

## Open questions

- Which step duties can move from the agent to hooks, while decision 5 holds?
- How many patterns does the search budget show?
- Which `--json` keys change with the label?
