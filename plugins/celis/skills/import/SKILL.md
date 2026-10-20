---
name: import
description: Imports a skill file, such as a SKILL.md, into celis as a named trail that work in every project can use by its name. Use when the person runs /celis:import <file or URL>, or asks to add a skill to celis.
---

You turn a skill file into a named trail in the project `skill:<name>`. Its rules hold for each ask
that uses it. The import is the ask: add no other ask for it.

1. Read the file the person gives, a path or a URL. Its name is the `name` of its frontmatter, else a
   short noun: a lowercase letter, then lowercase letters, digits and `-`.
2. Run `celis names`. When another trail holds the name, read it with `celis tree <its id> --full`. When
   it came from this file, go to step 6. Else ask the person for another name.
3. Split the file into rules and steps:
   - A rule for each thing the work must keep or avoid: "When <what you see>: <what to do>, because
     <the reason>. (<the file's section>)". Copy the meaning, not the words, and keep the reason the
     file gives.
   - A step for each part of the procedure, in order: a verb and what it acts on.
4. Show the person the name, each rule and each step. Write nothing before they say yes.
5. Write the named trail as `steps.md` says, under "Named trails". Each step's proof is
   "From <file>, <section>.", and the ask's proof is "Imported <n> rules and <n> steps from <file>."
6. To import the file again, add a new ask under the same trail with
   `celis step add --project skill:<name> --under <trail id> -- "Update <name> from <file>"`. Compare each
   section with the rule that names it. Write a changed rule with `--replaces <its id>`, and a new one
   without. Leave each rule whose section is the same. Tell the person each section the file no longer
   has: its rule stays. Show the changes, write them after the person says yes, then close the ask.
7. celis prints each override that a changed rule ends. Tell the person: the project that wrote it now
   keeps the new rule.
8. Tell the person the name, the trail id, how many rules and steps it holds, and that
   `/celis:use <name> <ask>` uses it.
