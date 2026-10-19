---
cip: 7
title: Names
status: draft
depends: [CIP-01, CIP-03, CIP-04, CIP-05, CIP-06]
---

# CIP-07 — Names

A name makes a trail a skill: work in any project can use it by its name, keep its rules, and override
them in its own project.

## Names

```json
{ "kind": 1985, "content": "<name>",
  "tags": [["L", "celis.step"], ["l", "name", "celis.step"], ["e", "<trail id>"]] }
```

- content: the name: a lowercase letter, then lowercase letters, digits and `-`.

**Writers**

- A writer names only a trail.
- A writer MUST NOT write a name that a trail already holds.

**Readers**

- The oldest name record of a name, then the one with the lower id, MUST hold the name. A reader MUST
  ignore the others, and a name record whose content is no name.
- A trail with a name is **named**.

## Skill projects

```
skill:<name>     the project id of a trail written from a skill file, outside any repo
```

**Writers**

- A writer MUST write the skill `<name>` into the project `skill:<name>`, with `<name>` as its project
  name.

## Rules of a name

**Readers**

- A step that uses (CIP-06) a step of a named trail MUST keep the rules of that trail (CIP-04, "Rules"),
  and the rules of each named trail that a step of its tree uses, at any depth, each trail once.

## Overrides

A rule that replaces (CIP-05) a `trail:<id>` rule of another project is an **override**.

**Readers**

- An override MUST hide the rule it replaces only in its own project. Elsewhere that rule stays live.
- An override MUST NOT hold once a rule of the replaced rule's own project replaces that rule.
