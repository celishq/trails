---
cip: 1
title: Decisions
status: draft
depends: []
---

# CIP-01 — Decisions

A decision is one sentence that holds in its scope until another replaces it. An agent writes it when it
learns a rule that later work must keep. This CIP lets a reader know which decisions hold at a place, in
a trail, in a project and in every project. It also defines the store, the project and the account that
every other CIP uses.

## The store

The store holds the records of every project on this machine. Every agent on the machine shares it.

```
$CELIS_HOME/trails/
  shared/<created_at>-<id>.json    one record, as NIP-01 JSON
```

**Meaning**

- A writer and a reader MUST use the store in `$CELIS_HOME/trails/`, whatever the working folder.
- A writer MUST write a record to a file named `<created_at>-<id>.json`, holding the record as NIP-01
  JSON. It MUST create the store when there is none.
- A record file MUST NOT change once written. A reader MUST NOT see a file that is half written.
- A reader MUST skip a file that does not parse or whose record does not verify.
- A reader MUST read no store as an empty store, and MUST create nothing.

## Projects

A project is the code an agent works on. Every step and decision names its project.

```
project id     the first commit of the git repo that holds the working folder: the first of
               `git rev-list --max-parents=0 HEAD`, sorted; else the path of the top folder of the
               repo or worktree; else the path of the working folder
project name   the name of the main checkout's folder; else of the working folder
```

**Meaning**

- Every clone and every worktree of one repo MUST give the same project id.
- The name MUST NOT change the project id.

**Examples**

Agent A works in `~/code/app`, and agent B in its worktree `~/code/app/.claude/worktrees/feat-b`. Both
have the project id of the repo's first commit, and the name `app`. Agent C works in `~/code/shop`. All
three share one store; C reads only the decisions of `shop`.

> **Why the first commit.** A path changes from clone to clone and from worktree to worktree. The first
> commit does not, so a later sync between machines keeps the same project.

## The account

Every record is signed by the account. The phrase gives the account.

```
phrase    $CELIS_HOME/phrase: 12 BIP-39 words, minted on first use when it does not exist
account   the key the phrase gives (`celis-nostr`)
```

**Meaning**

- A writer MUST write `$CELIS_HOME/phrase` readable by its owner alone. Two writers that mint at once
  MUST end with one phrase.
- A phrase MUST give the account the Celis app gives for it.
- A writer MUST mint a phrase only when it writes a record.
- A writer MUST sign every record as the account. An agent that works for you signs as you.
- A record MUST NOT carry the phrase or the account's secret.
- A reader MUST count every record that verifies, whatever account signed it.

> **Why every signer counts.** Records from your other accounts, and later from other people over the
> wire, land in one store. A record the store holds is one you chose to keep.
>
> **Why a file.** The phrase is kept as `$CELIS_HOME/key` was, so the command works the same on every
> system. The Keychain can hold it later.

## Decisions

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

```
scope      where the decision holds ("Scope")
replaces   the decision this one takes the place of
```

**Meaning**

- A decision MUST carry content, the anchor `project:<project id>` and the project name.
- A reader MUST ignore cairns (kind 1111 in `celis.cairn`), ratings (kind 7), outcomes (`celis.outcome`),
  source records (`celis.source`) and splits a store holds from before.

**Derived**

- A decision is live until another decision names it in `replaces`.

**Examples**

```
"When you change shelf/prices.py: keep prices in cents, because floats lost a cent at checkout."
```

> **Why no score.** A score ordered every cairn of every project, so an agent could read them all. Read by
> place and trail, few decisions apply, and a wrong one is replaced, not voted down.

## Scope

A scope is narrow or wide, so a rule for one file and a rule for all your work can live in one store.

```
at:<place>        at one place of its project: a file, a file:line, a folder or a URL
trail:<step id>   in its project, while that trail is active (CIP-04)
project           in its project; a decision with no `scope` tag
me                in every project
```

**Meaning**

- A writer MUST write the whole id of the trail that `trail:<id>` names.
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

A place is a file, a `file:line`, a folder or a URL. A record holds a path from the project's top folder,
so `./a.rs`, a path from a subfolder and the full path name one place.

**Meaning**

- A writer MUST resolve a place that is a path against the working folder, and drop its `.` and `..`.
- A writer MUST write that path from the project's top folder. A `:<line>` MUST stay at its end.
- A writer MUST keep a URL, and a path outside the project's top folder, as it was given.

**Examples**

```
in ~/code/shop/src:   ./prices.py:40                         →   src/prices.py:40
in ~/code/shop:       /Users/me/code/shop/src/prices.py      →   src/prices.py
```

## Reading

A reader gives the live decisions in scope, narrow first.

**Meaning**

- A reader MUST give only live decisions in scope.
- A reader MUST order decisions by scope, `at` first, then `trail`, then `project`, then `me`. In one
  scope, it MUST order them newest first, then by id.

**Derived**

- At a place: each live decision with `at:<place>` at the place, under it when the place is a folder,
  or at a folder that holds the place.
- In scope: every live decision of the working folder's project, and every live `me` decision.
- A decision at a file is at that file whatever line its place names.
- A decision at a folder is at each place under that folder.
