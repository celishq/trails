---
cip: 10
title: Decisions
status: draft
depends: [CIP-01]
---

# CIP-10 — Decisions

## The store

The store holds the trails, steps and decisions of every project on this machine. Every agent on the
machine shares it.

```
$CELIS_HOME/trails/
  shared/<created_at>-<id>.json    one record: a step, a done, a steer, a decision

$CELIS_HOME   CIP-11, "Commands"
```

**Rules**

- A command MUST open the store in `$CELIS_HOME/trails/`, whatever the working folder.
- With no store, `read` and `tree` MUST answer as an empty store, and write nothing.
- With no store, every other command MUST create it.
- A record MUST be written to a file named `<created_at>-<id>.json`, holding the record as NIP-01 JSON.
- A record file MUST NOT change once written. A reader MUST NOT see a file that is half written.
- A reader MUST skip a file that does not parse or whose record does not verify.

## Projects

A project is the code an agent works on. Every step and decision names its project.

```
project id     the first commit of the git repo that holds the working folder: the first of
               `git rev-list --max-parents=0 HEAD`, sorted; else the path of the top folder of the
               repo or worktree; else the path of the working folder
project name   the name of the main checkout's folder; else of the working folder
```

**Rules**

- Every clone and every worktree of one repo MUST give the same project id.
- The name MUST NOT change the project id.

**Examples**

Agent A works in `~/code/app`, and agent B in its worktree `~/code/app/.claude/worktrees/feat-b`. Both
have the project id of the repo's first commit, and the name `app`. Agent C works in `~/code/shop`. All
three share one store; C reads only the decisions of `shop`.

> **Why the first commit.** A path changes from clone to clone and from worktree to worktree. The first
> commit does not, so a later sync between machines keeps the same project.

## The key

```
account   $CELIS_HOME/key (CIP-01), minted at random on first use when it does not exist
```

**Rules**

- The engine MUST write a minted `$CELIS_HOME/key` readable by its owner alone. Two commands that mint at
  once MUST end with one key.
- A record MUST count only when the account of `$CELIS_HOME/key` signed it.
- Every command MUST sign as the account. An agent that works for you signs as you.

## Decisions

A decision is one sentence that holds until another replaces it. An agent writes it, signed as your
account, when it learns a rule that later work must keep. Each holds in its scope.

```json
kind 1111, signed by the account (NIP-22)
{ "kind": 1111,
  "tags": [
    ["I", "project:<project id>"], ["K", "project"],
    ["i", "project:<project id>"], ["k", "project"],
    ["project", "<project name>"],
    ["scope", "<me | trail:<step id> | at:<place>>"],     optional: with none, the project
    ["replaces", "<id of the decision it replaces>"],     optional
    ["L", "celis.decision"], ["l", "decision", "celis.decision"]
  ],
  "content": "When <what you see>: <what to do>, because <why>." }
```

**Rules**

- A decision MUST be live until a counted decision names it in `replaces`.
- A reader MUST ignore cairns (kind 1111 in `celis.cairn`), ratings (kind 7), outcomes (`celis.outcome`),
  source records (`celis.source`) and splits a store holds from before.

**Examples**

```
"When you change shelf/prices.py: keep prices in cents, because floats lost a cent at checkout."
```

> **Why no score.** A score ordered every cairn of every project, so an agent could read them all. Read by
> place and trail, few decisions apply, and a wrong one is replaced, not voted down.

## Scope

A decision holds in its scope. A scope is narrow or wide, so a rule for one file and a rule for all your
work can live in one store.

```
at:<place>        at one place of its project: a file, a file:line, a folder or a URL
trail:<step id>   in its project, while that trail is active (CIP-14)
project           in its project; a decision with no `scope` tag
me                in every project
```

**Rules**

- A reader MUST count a decision with no `scope` tag as `project`.
- A reader MUST give a `me` decision in every project. It MUST give any other decision only in its own
  project.
- A reader MUST skip a decision whose `scope` it does not know.

**Examples**

```
at:shelf/prices.py   "When you change prices: keep them in cents, because floats lost a cent."
me                   "When you write a commit message: use plain words, because the team reads them."
```

## Places

A place is a file, a `file:line`, a folder or a URL. A command writes and reads a path from the project's
top folder. So `./a.rs`, a path from a subfolder and the full path name one place.

**Rules**

- A command MUST resolve a place that is a path against the working folder, and drop its `.` and `..`.
- A command MUST write and read that path from the project's top folder. A `:<line>` MUST stay at its end.
- A command MUST keep a URL, and a path outside the project's top folder, as it was given.

**Examples**

```
in ~/code/shop/src:   --at ./prices.py:40    →   src/prices.py:40
in ~/code/shop:       --at /Users/me/code/shop/src/prices.py   →   src/prices.py
```

## Reading

A reader gives the live decisions in scope, narrow first.

```
at a place   a decision with `at:<place>` at the place, under it when the place is a folder, or at a
             folder that holds the place
in scope     every live decision of the working folder's project, and every live `me` decision
```

**Rules**

- A reader MUST give only live decisions in scope.
- A decision at a file MUST be at that file whatever line its place names.
- A decision at a folder MUST be at each place under that folder.
- A reader MUST order decisions by scope, `at` first, then `trail`, then `project`, then `me`. In one
  scope, it MUST order them newest first, then by id.
