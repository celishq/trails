# celis — design

This page keeps the design decisions and why we took them. The contract is in `spec/`, and the code
follows the spec. A change of a decision changes this page first, then the spec, then the code. A plan
names the decisions it acts on.

## The core

celis keeps the knowledge that agents need to do work well, and the evidence for it.

- **Patterns are the core.** A pattern is a piece of knowledge: in a context, choose this, because of
  that. It activates in each trail whose context matches it.
- **Rules are correctness.** A rule is a pattern with `binds`. A result must keep it.
- **Patterns are quality and taste.** A pattern guides, and work may depart from it with a named reason.
  Pairs show taste: good and bad examples, made from the person's picks and corrections.
- **Trails are the evidence.** A trail records an ask, its steps and their proofs. It gives observability,
  and it is the feedback loop that shows whether a pattern is still useful: its examples, its uses, its
  recalls, the judge and tidy.

The store, the query, the card's method, the log, tidy and the tree are the core, because no model can
know them by itself. Hooks, ranks and caps are a scaffold, which a smarter model can outgrow.

## Decisions

1. **A pattern holds in a project or for me, never in one trail.** The `trail:<id>` scope goes. What
   holds for one trail only is a note on its step. A reader reads an old trail pattern as a note of
   that trail.
2. **A rule and a pattern are one record.** The `binds` label splits them. The wire names stay:
   `celis.decision`, the `decision` label and the `--json` keys.
3. **Search finds every rule, and the best patterns.**
   - A rule is never cut from the output. It gets a narrow trigger, such as a `path`.
   - Patterns are ranked by how well they fit, and the output shows only a budget of them.
   - The past of like asks is a helper part, and it is short.
4. **What goes in matters more than search.** An agent writes a pattern only for a choice that will come
   back. It replaces a pattern instead of adding a near copy.
5. **The agent writes each record with a celis call.** A hook may give the agent context, such as the
   patterns that fit, but it never captures the agent's tools to write records.
6. **Patterns lead.** The card, the help and the README start with patterns, and trails come after.
7. **Names follow the core.** The crate `celis-trails` becomes `celis-patterns`. The repo keeps its name
   for now, and becomes `celis` later.

## What we do not do

- No new record kind for rules, and no new wire names for old ones.
- No rank or grade of work by time.
- No sharing across a team or an org yet.

## Open questions

- Which step duties can move from the agent to hooks, while decision 5 holds?
- How many patterns does the search budget show?
- Which bench proves a search change: the `--uses` links and the judge's citations as an offline set?
