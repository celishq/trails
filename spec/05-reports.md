---
cip: 5
title: Reports
status: draft
depends: [CIP-01, CIP-02, CIP-03, CIP-04]
---

# CIP-05 — Reports

A report gives the work of trails to a person outside the store: in a pull request, or to the celis team
when something went wrong. This CIP lets a reader know what a report of a trail holds, what the log of
failed runs holds, and what an issue report holds. A report is no record. CIP-02 gives their layouts.

## The report

A report holds the trails the agent names, so a person can add the work to a pull request.

**Derived**

- Of a trail: its title, id, state (CIP-04, "State"), how long its tree ran, the count of steps under it,
  of its rules, of its revised steps and of its dropped steps, and its project. Then the lines of its
  text after its title, and its proof when done.
- Then the steps under it, oldest first, each followed by its own: its state, its title, how long it took
  when done, and its proof and what changed when done, or its reason when dropped.
- Then of each step with any: the lines of its text after its title, its places and its check. A done
  step's check passed, because a `done` is written only after it passes (CIP-03, "Closing").
- Then its rules (CIP-04, "Rules").
- When a trail's first ask has the trail's text, the ask stands for the trail. Its steps come first, then
  the trail's other asks.

**Meaning**

- A report MUST only read the store. It MUST NOT run a check.

## The log

The log keeps the runs of `celis` that failed, so a person can show the celis team what went wrong. It
leaves out the text a person gives, because that text holds their work.

```
$CELIS_HOME/log
  <yyyy-mm-dd> <hh:mm> <exit code> celis <words> · <message>      one line a failed run
```

**Meaning**

- A run of `celis` that fails MUST add one line to `$CELIS_HOME/log`: the local time, the exit code,
  `celis` with the words of the command, and the error message.
- The words MUST leave out the flags, their values and the text after `--`.
- A run that succeeds MUST NOT write the log. A run MUST NOT fail because it cannot write the log.

**Examples**

```
2026-10-13 14:02 1 celis step done ffffffff · no step has the id ffffffff; find it with `celis tree --all`
```

## The issue

An issue report gives the celis team what they need to find a fault. The agent writes what went wrong,
shows the report to the person, and sends it only the way the person chooses.

**Derived**

- What went wrong, in the agent's words.
- The facts: the version as `celis --version` prints it, the OS and CPU, the names of the session
  variables that are set (CIP-04, "Sessions"), the working folder's project, `$CELIS_HOME` with the home
  folder as `~`, whether the phrase exists, and the count of record files in the store, of those that
  read, and of steps.
- The last 50 lines of the log.
- The trails the agent names, as a report holds them.

**Meaning**

- An issue report MUST NOT hold the phrase. It MUST only read the store.
