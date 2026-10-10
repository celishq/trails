---
cip: 14
title: Trails
status: draft
depends: [CIP-10, CIP-11, CIP-12]
---

# CIP-14 — Trails

## Trails

A trail is one task you give, in your words. It is a step with no `under` (CIP-12, "Steps"). The steps
under it, and the steps under those, are its tree.

**Rules**

- A reader MUST count a step with no `under` as a trail.
- A reader MUST count a step in the tree of the trail at the top of its `under` chain.

**Examples**

```
"The price list shows a senior tier at 80% of the price, and the receipt shows it."
"Make the badges orange."
```

## Rules

A trail's rules are the live decisions with the scope `trail:<id>` (CIP-10, "Scope"). Each is one
sentence a check can fail.

**Rules**

- A reader MUST count a decision as a rule of a trail only when its scope names that trail.

**Examples**

```
celis decide --scope trail:4c1d22e0 -- "A senior pays 80% of the price, rounded down to the cent."
```

## State

A trail has no state record. Its state comes from its tree, so the person can see which tasks the agent
reached alone.

```
active              the trail is open
reached alone       the trail is done, and no steer names a step of its tree
reached with help   the trail is done, and a steer names a step of its tree
```

**Rules**

- A reader MUST work out a trail's state from its tree, their `done` records and their steers.

## The commands

```
celis tree [<step id>]   a trail, its rules and its tree; with no id, every trail
```

**Rules**

- `tree` MUST take a step id or its first 8 characters, as CIP-11 "Output and errors" states. It MUST
  refuse an id no step has, with `no-step`.
- `tree` with no id MUST print every trail of the working folder's project, newest first.
- `tree` MUST print the step with its state, its proof and grade when done, the trail it follows, and its
  rules. Then come the steps under it, oldest first, each followed by its own steps.
- `tree` MUST mark a done step `✓`, a working one `◐`, and an open one `○` (CIP-12, "Starting"). Under a
  done step come its proof and its grade (CIP-12, "Grades"). `tree` MUST NOT write a record.

```
tree  text  <id, first 8> [<project name>] <text> · <active | reached alone | reached with help>
                done <proof>
                <its grade>
              follows <id, first 8> <the first line of that trail>
              rule <id, first 8> <text>
            <mark> <a step, as CIP-12 "The commands" gives it>    two spaces more for each level
                done <proof>
                <its grade>
      json  {"trail": {"id", "created_at", "project", "content", "state", "follows", "proof", "grade"}}
            · {"step": {<as CIP-12 "The commands" gives it>, "state": "done | working | open", "proof",
            "grade"}}, one a line
```

**Examples**

```
celis tree 3fa2c1d0
```

## The past

The past shows how a task like a new one was done, so a new plan can follow the path that worked.

**Rules**

- `read --like` MUST print the done trails of every project that share a word with the text, 3 at most.
  It MUST print each as `tree` prints it. With `--json`, the trail's object is named `past`.
- The most shared words MUST come first, then the fewest steers and rework in the tree, then the newest.
- A word MUST be 4 or more letters or digits, in lower case.
- `step add --follows <id>` MUST write a trail that follows the past trail `<id>`. It MUST refuse an id
  no trail has, with `no-trail`. With `--under`, it MUST refuse as a usage error.

```
celis read --like "Add an Invoices page from the frame"
celis step add --follows 3fa2c1d0 -- "Add an Invoices page from the frame."
```

## Watching

`watch` shows a trail while the agents work on it, so the person sees each step start and close.

```
celis watch [<step id>]   a trail's tree, printed again on each change to the store
```

**Rules**

- `watch <id>` MUST print the tree of the step `<id>` as `tree` prints it. It MUST refuse an id no step
  has, with `no-step`.
- `watch` with no id MUST print the newest active trail of the working folder's project. When a newer
  trail starts, it MUST print that one. With no active trail, it MUST print nothing until one starts.
- `watch` MUST print the tree again within one second after a record lands in the store, and only when
  the tree changed.
- On a terminal, `watch` MUST clear the screen and its scrollback, and print each tree from the top
  left. Else it MUST print an empty line between two trees.
- `watch` MUST run until it is stopped, and MUST NOT write a record.

**Examples**

```
celis watch 3fa2c1d0
```
