# Celis core redesign: rules that come at the right moment

## Goal

Agents work better with celis when they get the few rules that the current moment needs, at that moment,
and when the person can see the work. Today celis gives too many rules at the start of an ask and finds
them by the words of the ask. Recall becomes worse as the store grows.

## Evidence

All numbers come from `evals/drift` (8 tasks, 1 run each arm) and from a copy of the real store, on
2026-10-20 and 2026-10-21.

| Arm                                  | 400 rules, Haiku | 400 rules, Opus | 1000 rules, Haiku | 1000 rules, Opus |
| ------------------------------------ | ---------------- | --------------- | ----------------- | ---------------- |
| celis today (grep by the past agent) | 75% broken       | 62%             | 85%               | 70%              |
| pull (the agent searches by itself)  | 71%              | 49%             | —                 | —                |
| pickers (Haiku reads all rules)      | 35%              | 33%             | 45%               | 43%              |
| triggered (hook at each action)      | 22%              | 26%             | 61%               | 62%              |

After the build (2026-10-21, the same tasks, 1 run each arm):

| Arm                                      | 400 rules, Haiku | 400 rules, Opus | 1000 rules, Haiku | 1000 rules, Opus |
| ---------------------------------------- | ---------------- | --------------- | ----------------- | ---------------- |
| triggered on `celis rules` (cap 5, rank) | 29% broken       | 41%             | 60%               | 61%              |
| taught (the card and the query, no hook) | 64%              | 41%             | 74%               | 48%              |

Without hooks (2026-10-21, the same tasks, 1 run each arm; broken, then old in brackets):

| Arm                                                     | Haiku 400 / 1000  | Sonnet 400 / 1000 | Opus 400 / 1000   |
| ------------------------------------------------------- | ----------------- | ----------------- | ----------------- |
| delegated (the past agent runs `celis rules` at the ask) | 87% / 79%         | 48% / 51%         | 79% / 86%         |
| coordinated (the agent searches itself: index, `celis grep`, `celis rules` before each change) | 71% / 59% | 24% / 48% | 23% (0%) / 42% (15%) |
| reminded (coordinated, with notes at step add and step start)                    | —                 | 22% (3%) / 39% (17%) | 23% (3%) / 32% (4%) |
| pickers                                                 | 35% / 45%         | 30% / 41%         | 33% / 43%         |

The Sonnet numbers above count every check. With the plain Sonnet run, the report now leaves out the 10
habits Sonnet keeps with no memory, as it does for Haiku and Opus. Broken, then old, on that count:

| Arm (Sonnet)                                                       | 400 rules  | 1000 rules |
| ------------------------------------------------------------------ | ---------- | ---------- |
| plain (no memory)                                                  | 96% (100%) | —          |
| celis today (main's plugin and the installed celis)                | 69% (59%)  | 77% (61%)  |
| delegated                                                          | 73% (63%)  | 76% (29%)  |
| coordinated                                                        | 36% (27%)  | 73% (48%)  |
| reminded                                                           | 33% (3%)   | 58% (17%)  |
| sized (reminded, no past agent, content rules wait for text, --diff) | 66% (39%)  | 64% (43%)  |
| pickers                                                            | 45% (50%)  | 61% (68%)  |

On Opus, sized broke 26% (4%) and 38% (11%), against reminded's 23% (3%) and 32% (4%).

- With no past agent, Sonnet added no step in 4 to 5 tasks of 8: it searched, then did the work with no
  `step add` and no `step start`, so it got no note and checked nothing. With the past agent, it added
  and started a step in all 16 tasks. The call of the agent held Sonnet in the method; the search itself
  was not what helped.
- Opus kept the method with no past agent and ran `celis rules --diff` in 8 to 9 tasks of 8. Its breaks
  are rules it saw.

Tooled (reminded with the past agent, `celis rules --diff` named for the check, and a size line and
`--page` past 16 KB) broke Sonnet 52% (32%) / 66% (28%) and Opus 36% (21%) / 52% (30%): worse than
reminded on both.

- Sonnet cut `celis rules` with `head` or `tail` in 35 of 44 calls, against 19 of 43 in reminded. The
  size line says that the output is long, and `tail` cut that line away with the first rules. `--page`
  came 5 times in 32 tasks.
- The check with `--diff` gave the text of only the rules not given before. The rest came as one line of
  ids, so the agent checked its change against ids.
- The real-rules bench (5 new tasks on celis-trails, a copy of the real store, an Opus judge over 426
  rules) found 4 of 37 applying rules broken with the base plugin, 3 of 35 with the new one, and 4 of 37
  with no memory. Five small tasks cannot tell the arms apart.

One change per arm (2026-10-21, Sonnet and Opus, 400 / 1000 rules, broken):

| Arm                                              | Sonnet      | Opus        |
| ------------------------------------------------ | ----------- | ----------- |
| reminded                                         | 33% / 58%   | 23% / 32%   |
| full (every rule in full on each call, no skip)  | 39% / 65%   | 27% / 44%   |
| checked (full, with `celis rules --diff` named)  | 31% / 66%   | 19% / 34%   |

The real-rules bench, 15 tasks on Opus, 11 valid (4 named code only on this branch): base broke 7 of 58
applying rules, checked 6 of 67, no memory 7 of 62. Most breaks are tasks that ask for what a rule
forbids, where the ask wins; all three arms broke the same few. The bench cannot tell the arms apart.

Each part of checked removed in turn, at 400 rules, 2 runs a cell (rules broken, run 1 / run 2):

| Arm                                                     | Sonnet            | Opus              |
| ------------------------------------------------------- | ----------------- | ----------------- |
| checked                                                 | 34% (31% / 38%)   | 18% (19% / 16%)   |
| quiet (no note at step add or step start)               | 43% (41% / 45%)   | 21% (20% / 22%)   |
| nodiff (`celis rules` on the files git names, no `--diff`) | 45% (45% / 45%) | 23% (21% / 24%)   |
| words (no triggers: the agent searches by words)        | 54% (55% / 53%)   | 48% (49% / 47%)   |

- Every removal broke more rules. Triggers matter most: without them, both models break 20 to 30 points
  more, and keep the old version of a replaced rule 14% to 30% of the time, not 2% to 7%.
- The notes and `--diff` cost Opus only 3 to 5 points, near the run spread, but Sonnet 9 to 11. The
  design keeps all three parts.

- With the notes, Sonnet searched the rules with `celis grep` in 6 to 7 tasks of 8, not 1, and ran
  `celis rules` after a write 7 times, not 0 or 1. It beat pickers at 400 and 1000 rules.
- Sonnet still lost 55 rules at 1000 rules that it never saw: the harness cut 5 outputs of `celis rules`
  to a 2 KB preview. A file not yet made matches every content rule by its path alone.

- The agent that searches itself is the best arm on Opus at 400 rules and ties pickers at 1000. Sonnet
  beats pickers at 400 and loses at 1000. Haiku stays far behind pickers.
- Opus read the index and searched with `celis grep` in all 8 tasks at 1000 rules. Sonnet and Haiku did
  it in 1 or 2 tasks of 8, and ran `celis rules` on the files only.
- Haiku cut the output of `celis rules` with `head` in 10 of 18 calls at 1000 rules, though the card
  says not to.
- No task gave work to a subagent, so the drift bench does not test the rules passed to subagents.
- The first `celis rules` on a new Python file gives 162 rules (17 KB) at 400 rules. A later call in the
  session gives only the new ones, and the ids of the rest (2 KB). The index is 3 KB.

- Each broken rule of the triggered arm at 400 rules was a rule the agent never got. One Python write
  matches about 100 drift rules, and the cap gives 5 a call, so a session got 60 to 150 of them. The
  old hook gave about 9000 characters a call (40 to 60 rules).
- With the rank, triggered kept no old version of a replaced rule on Opus (0% at 400 and 1000).
- On Opus at 1000 rules, the taught arm broke fewer rules than triggered (48% to 61%).
- Raycast, Sonnet, 1 run: the new plugin broke 2% of 43 rule checks, today's plugin 7%.

- At 400 rules, triggered kept the old version of a replaced rule 4% of the time. The other arms kept it
  11% to 46% of the time.
- At 1000 rules, triggered gave about 300 rules a task. All drift rules are Python conventions, so all of
  them apply to each write. Triggers tell when a rule applies, but not which few rules matter.
- In the pull arm, the agent searched 1 to 3 times, with the words of the ask, and cut the output with
  `head`. An agent cannot search for a rule it does not suspect. Opus broke 22 points fewer rules than
  Haiku, so a smarter agent pulls better.
- The real store has 932 live rules: 65% of them a file shows, 9% a command, 4% another tool call, 16%
  only a situation, and 5% are one-off notes.
- Triggers on the 884 real rules, replayed on 15104 real tool calls, gave a median of 2 rules a call
  (p90 4 to 14, max 66), and 12 to 125 rules in a long session.
- 7 of 1596 real steps follow a past step. Step reuse does not work yet.
- The bench code is in `evals/drift`: `triggers.py`, `trigger.py`, and the `triggered` and `pull` arms
  of `run.py`. The store copy and the real-rule data are in a scratchpad. They are not in the repo.

Known weak points of this design, which the phases below address:

| Weak point                                                          | Where it is addressed                |
| ------------------------------------------------------------------- | ------------------------------------ |
| A rule comes at the write, after the agent has made its plan        | Phase 3: a hook before Read too      |
| "Once a session" loses the rules when the context is compacted     | Phase 1: compaction resets it        |
| The cap hides rules with no sign                                    | Phase 1: a pointer to the rest       |
| A rule that did not come is a silent miss                           | Phase 5: misses in the log and tidy  |
| The rank by corrections is a guess, because corrections are rare   | Phase 1 and 5: a fixed order first   |
| The drift bench is one kind of rule (Python conventions)            | Phase 3: the Raycast bench too       |
| Other agents have no full hooks                                     | Phase 2: the card teaches the query  |

## Design

```
                           ┌───────────────────────────────────────────┐
   the person  ───talks──▶ │                  AGENT                    │
       ▲                   │   smart today, smarter tomorrow           │
       │                   └──┬──────────────┬──────────────┬──────────┘
       │ sees the work        │ writes       │ asks         │ is given
       │                      ▼              ▼              ▲
╔══════╧══════════════════════════════════════════════════════════════════╗
║  DURABLE CORE: what no model can know by itself                         ║
║                                                                         ║
║   STORE    steps · proofs · rules (When X: do Y, because Z)             ║
║            · triggers · corrections (replaces) · drops                  ║
║            signed records, across sessions, projects and agents         ║
║                                                                         ║
║   QUERY    celis rules <call>    the rules of a file, command or tool   ║
║            celis grep / tree     depth and past work                    ║
║                                                                         ║
║   CARD     the method: ask before you act → one step per ask → prove    ║
║            it → write what surprised you → replace what was wrong       ║
║                                                                         ║
║   LOG      rules given, with step and session · misses                  ║
║                                                                         ║
║   TIDY     merge · drop · fade · fix triggers that miss                 ║
║            · move rules into linter / tests / CI / AGENTS.md            ║
║                                                                         ║
║   TREE     what was done, proven and given (the person's view)          ║
╚═════════════════════════════════════════════════════════════════════════╝
                ▲  uses only the core, adds nothing new
┌───────────────┴─────────────────────────────────────────────────────────┐
│  SCAFFOLD: helps today's models, shrinks over time                      │
│                                                                         │
│   PUSH   hooks before Read / Write / Edit / Bash / tools, after Bash    │
│   PICK   subagent at the ask: ≤10 situation rules + past steps          │
│   RANK   correction → misses → narrow trigger                           │
│   CAP    5 a call · 10 at the ask · "N more" pointer                    │
└─────────────────────────────────────────────────────────────────────────┘
```

How the parts change over time:

```
                  today            next models          very smart agent
                  ─────            ───────────          ────────────────
 PUSH (hooks)     ███████████      █████                ░  off, or a light net
 PICK (subagent)  ███████████      ███████              ░  the agent picks itself
 RANK + CAP       ███████████      █████                ░  the agent judges
 CARD (method)    ████             ███████              ███████████  main guide
 QUERY            ████             ███████              ███████████  main road
 STORE · LOG      ███████████      ███████████          ███████████  never changes
 TREE · TIDY      ███████████      ███████████          ███████████  never changes

 the gate:  taught arm (card + query, no hooks)  vs  triggered arm (hooks)
            run it on each new model → when they are close, turn the scaffold off
```

| Lasts                | Because                                                               |
| -------------------- | --------------------------------------------------------------------- |
| The store            | What the person and past sessions decided is in no model's training.  |
| Triggers as an index | "The rules of this file" is one exact call, not a guess at words.     |
| The method           | It is how good engineers work, at any level of skill.                 |
| The log and misses   | They are the facts of what happened.                                  |
| The tree             | The person must see the work, more so as the agent gets better.       |
| Tidy                 | A store that only grows becomes noise for any agent.                  |

The scaffold uses only the core. When a model keeps the rules as well with the card and the query as
with the scaffold, the scaffold can go off for that model, in a later release.

## How the agent works

The agent is the coordinator, on Sonnet or a larger model. It finds the rules itself and keeps them in
its own context. celis says what to do at the moment the agent needs it, because the card at the session
start is far from the work and the agent skips calls it only read about.

```
moment          celis says                                        the agent does
--------------  ------------------------------------------------  ------------------------------------------
new prompt      prompt hook: where to add the ask; run celis:past  runs celis:past for past asks
step add (ask)  find the rules: celis rules, then celis grep       reads the index, greps the words that fit
step start      celis rules <files> before each change; check the  gets the rules of each file before it
                change against them before done                    changes it; checks its change before done
step done       the review, and the next move                      writes a rule for what surprised it
subagent        (card) put the rules of the work in its task        passes the rules; the subagent searches nothing
```

- celis prints help and the rules the plan links (trail rules, used rules, named trails). It never picks
  rules by words or by place for the agent: the agent asks `celis rules` and `celis grep` for them.
- A rule a tool can check belongs in that tool. tidy proposes the move.

## Interface

The phases share this contract. Phase 1 writes it into the spec.

```
rule tags (CIP-05)   ["path", "<glob>"]...        a file the rule acts on, from the project's top folder;
                                                  * within a segment, ** across segments, ? one character
                     ["content", "<regex>"]       what the file's text must hold
                     ["command", "<regex>"]       the shell commands it acts on
                     ["tool", "<regex>"]          the names of other tool calls it acts on
                     ["l", "situation", "celis.decision"]   no call shows it ahead
                     ["l", "promoted", "celis.decision"]    a tool of the place now checks it

decide               --path <glob>... --content <regex> --command <regex> --tool <regex> --situation
                     --promoted
rules                celis rules [<file>]... [--all] [--session <id>] [-- <command>]
                     with no file and no command: a hook's JSON on stdin
                     prints <id8> <scope> <rule> a line, at most 5, then
                     "celis: <n> more rules match ..." when more match
grep view            a rule's fields end with " trigger" when it has a trigger tag, " situation" when
                     it has the situation label, and " promoted" when it has the promoted label
log                  $CELIS_HOME/given, one JSON object a line:
                     {"at", "session", "rule"[, "step"]}   a rule given
                     {"at", "session", "compact": true}    a compaction; rules given before it count
                                                           as not given
                     {"at", "session", "rule", "miss": true}   a rule the agent replaced in a session
                                                           that was not given to it (phase 5)
```

A regex is the syntax of the Rust `regex` crate: no lookaround and no backreferences. One that does not
compile matches nothing. An `at:<place>` rule counts as a rule with that place as its `path`.

Principles:

1. Attention is the limit, not memory. The design fixes how many rules the agent sees at one moment.
2. A rule is worth giving when the agent would break it without it. Evidence ranks the rules, not their
   words.
3. A trigger is an index of the rule, not the rule. The rule text stays general.
4. A rule that a tool can check belongs in that tool. Memory holds what no tool can check.
5. Agents are workers who do the tasks and teach each other. The agent writes the steps, the rules and
   the corrections. Hooks only give what is written: they write no record and detect no break.
6. A model pick runs in a subagent, never in a hook. So a hook ranks and caps with no model.
7. The store stays small through tidy, not through cuts at read time.
8. The caps are fixed for now: 5 a call, 10 at the ask. A later release can rebalance them.
9. The agent is smart and gets smarter. Celis gives what no model can know by itself: what the person
   and past sessions decided, and proof of the work. The core holds that; the push only helps today's
   models.
10. A miss must be visible. When the agent corrects a rule that it was not given, the log shows it.

## Phases

All phases ship together, in one release. Each phase is one independent piece of work with its own
check, so one agent can take it alone. Older stores still read right after the release.

Phases 2, 3, 4 and 5 need phase 1. Phase 6 needs phase 2. Phase 7 needs phase 5.

### Phase 1. Triggers and the query

Result: an agent can ask celis for the rules of the call it is about to make.

- Add to CIP-05 Rules the trigger tags of a rule (`paths`, `content`, `command`, `tool`, `situation`)
  and the `promoted` label. A pattern that does not compile matches nothing.
- `celis decide` takes the trigger.
- `celis rules` gives the live rules that one call matches: a file read or write, a Bash command and the
  files it names, the files that a Bash command changed, or another tool call.
- At most 5 rules come at one call, the rules with the narrowest trigger first (a named file before a
  glob, a glob with content before one with none). When more match, the output ends with how many more
  there are, and the same call gives the next 5.
- The grep view marks a rule with a trigger, a situation rule and a promoted rule.
- A local file beside the store logs each rule given, with the session and the time. It is not a
  record, and links do not carry it. Each rule comes once a session, and once again after the context is
  compacted: `celis prompt` logs the compaction.
- Verify: unit tests on `celis rules` for each kind of call, the order, the pointer, and compaction,
  with spec citations; `make verify` passes.

### Phase 2. The card: a method of robust work, with light steps

Result: the card teaches a smart agent how to work with celis, and a simple ask costs one step.

- The card says: before you act, ask celis for the rules of the files and commands you will touch, and
  for past asks like this one; write one step for each ask, and steps under it only when the ask is
  hard; close each step with a proof; when something surprises you, write the rule that would have
  saved you, with a narrow trigger; when you are corrected, replace the rule; where to look for depth
  (`celis rules`, `celis grep`, `celis tree`).
- On agents with no full hooks, the card is the only delivery, through `celis rules`.
- A new `taught` arm of the drift bench: the card and `celis rules`, with no hooks.
- Verify: `crates/celis/tests/help.rs` passes; the plugin evals are the same or better; a session of
  a simple ask shows one step in `celis tree`; the `taught` arm runs on Haiku and Opus at 400 and 1000
  rules, and its numbers go into Evidence.

### Phase 3. Hooks at the action

Result: a rule with a trigger comes right before the action it is about, with no ask from the agent.

- The Claude Code hooks call `celis rules` before Read, Write, Edit, Bash and other tool calls, and
  after Bash.
- Verify: the drift `triggered` arm on `celis rules` at 400 rules breaks no more than 26%; the Raycast
  bench breaks no more rules than today.

### Phase 4. Triggers for the rules already in the store

Result: the person's existing rules come at the moment too, and recall gets smaller.

- A `migrate` skill: the agent copies the store, writes a trigger for each live rule, and marks the
  one-off notes and the rules of done trails whose work was replaced. It shows the person the list,
  and writes to the store only after a yes: each rule with its trigger through `decide --replaces`, and
  a drop of each marked rule. A dropped rule leaves grep and recall, and the store keeps it for
  history.
- Verify: a dry run on a copy of the real store gives a trigger for each live rule and changes nothing;
  a real run leaves no live rule without a trigger or a situation mark.

### Phase 5. Evidence: the step, the misses and the rank

Result: `celis tree` shows which rules each step got, misses are visible, and the rank uses the
history.

- The log also holds the step that was working when a rule came.
- A miss: the agent replaces a rule that was not given in that session, because the cap cut it or its
  trigger missed. `celis decide --replaces` logs it.
- The rank in `celis rules`: a rule made or raised by a correction first, then a rule with misses, then
  the order of phase 1. A cut rule stays a candidate for the next call that matches.
- `celis tree` shows the rules given under each step.
- Verify: unit tests on the rank and on misses; drift at 1000 rules on Haiku and Opus breaks no more
  than pickers (45% and 43%), and at 400 rules no more than 26%. When the rank does not meet this, it
  ships off, and the order of phase 1 stays.

### Phase 6. The ask: situation rules and past steps

Result: at the ask, the agent gets at most 10 situation rules and the steps of past asks like it, in
place of all `me` and `project` rules.

- The `celis:past` subagent reads only the situation rules and the done asks. It gives at most 10 rules,
  each with its reason, and the steps of 1 or 2 past asks with the same goal, with their `changed`
  notes.
- The agent adds its steps with `--uses` for each past step it takes.
- Verify: an eval where a second ask repeats a first one shows a follow; the Raycast bench breaks no
  more rules than today; the count of follows in the real store goes up after a week.

### Phase 7. Tidy with evidence

Result: `/celis:tidy` proposes where rules belong, and fixes the triggers that miss.

- Tidy reads the log: it offers to fade rules given often and never corrected, to drop one-off notes,
  and to rewrite the trigger of each rule with misses.
- It proposes moves into the place's own checks (a repo's linter, tests or CI, a Figma project's styles)
  and into AGENTS.md. A moved rule stays in celis with the `promoted` label of CIP-05.
- Verify: eval `11-tidy` passes, with new cases for a proposal to a linter, to AGENTS.md, and for a
  trigger with misses.

## Not in this plan

- Team and org sharing over relays.
- Bootstrap from git history and PRs.
- Full hooks for Codex, Gemini CLI and Cursor. On them, the card teaches the query.
- The scaffold off for a model. A later release decides it from the `taught` arm.
- Rebalanced caps. The caps stay fixed until a bench or real use shows a better pair.
- Stats from the log. `celis stats` counts the rules an ask used from its `uses` tags. A later release
  makes it count the rules given from the log, and show the misses of each day.

## Open questions

None.
