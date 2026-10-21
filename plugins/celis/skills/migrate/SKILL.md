---
name: migrate
description: Gives the stored celis rules triggers, so that each rule comes right before the call it is about. Marks the rules that no call shows ahead, and drops one-off notes and rules of replaced work. Use when the person runs /celis:migrate, or asks to give their rules triggers.
---

You give each live rule a trigger or the `situation` mark, and drop each rule that no later work needs.
Write nothing to the store before the person says yes.

1. Ask the person what to migrate: this project, this project and the `me` rules, or all projects. Ask
   whether to do a dry run first, as "Dry run" says.
2. Add the migrate as one ask, as the card says.
3. List the live rules with `celis grep -E " rule [0-9a-f]{8} live "`. Skip each rule whose fields end
   with ` trigger`, ` situation` or ` promoted`, and each `at:<place>` rule: its place is its trigger.
   When a rule's text does not say enough, read its trail with `celis tree <trail id8>`.
4. Give each other rule a trigger, the `situation` mark, or a drop with why, as "Triggers" and "Drops"
   say.
5. Show the person the whole list: each rule's id8 and text, and its trigger, `situation` or drop with
   why. Ask: yes, no, or a change. With more than 50 rules, show and write them in batches of 50.
6. After a yes, write each rule of the list:
   - a trigger: `celis decide --replaces <id> [--scope <its scope>] [--path <glob>]... [--content <regex>]
     [--command <regex>] [--tool <regex>] -- "<the same text>"`;
   - `situation`: the same, with `--situation` in place of the trigger flags;
   - a drop: `celis step drop <id> -- "<why>"`.

   Keep the rule's scope, a done trail's too; `{project}` takes no `--scope`. To write into another
   project, add `--project <its id>`. The id is the `project` that `celis tree <a trail of it> --json` gives.
7. Run step 3 again: no rule must be left. Close the ask with the counts, and tell them to the person:
   rules given triggers, situation rules, drops, and live rules left with neither.

## Triggers

A trigger fires right before the call that the rule is about. The rule's text stays the same.

- Narrow beats broad. Name the files and the text the rule acts on: `--path "shop/prices.py"`, or
  `--path "**/*.py" --content "price|amount"`, not `--path "**"`. A trigger that fires at each call is
  noise.
- `--path` is a glob from the project's top folder: `*` within a segment, `**` across segments, `?` one
  character. Give one `--path` for each place.
- `--content` is a regex of the file's text, `--command` of the shell command, such as `^git commit`,
  and `--tool` of the name of another tool, such as `^mcp__figma__`.
- A regex is the syntax of the Rust `regex` crate: no lookaround and no backreferences. A pattern that
  does not compile matches nothing.
- A `me` rule holds in every project: its paths must hold in any project, such as `**/*.py`, never a
  path of one repo.
- Mark a rule `situation` when no file, command or tool call shows its case ahead, such as how to plan,
  or how to talk to the person.

## Drops

- A one-off note: a fact of one task that no later work needs, such as a port, a branch or a result.
- A rule of a done trail whose work a later step replaced: a step of its trail shows `revised-by <id>`,
  and the new work says otherwise.

A dropped rule leaves grep and recall, and the store keeps it for history.

## Dry run

A dry run does the migrate on a copy of the store, and leaves the store as it is:

1. `cp -R "$CELIS_HOME" <copy>`. With no `CELIS_HOME`, the store is `~/Library/Application Support/Celis`
   on macOS and `~/.local/share/celis` elsewhere.
2. `rm -f <copy>/links`, so that no write goes to a linked folder.
3. Do steps 2 to 7 with `CELIS_HOME=<copy>` before each `celis` command, and with no question in step 5.
   Show the person the list and the counts, then ask whether to migrate the store.
