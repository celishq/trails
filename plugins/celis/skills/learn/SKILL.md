---
name: learn
description: Drafts rules and patterns from a source, such as past chats, a skill file, a folder or a URL, and keeps the ones the person agrees to. Use when the person runs /celis:learn <source>, or asks celis to learn from their history, a document or a repo.
---

The text after `/celis:learn` names the source. You read it, draft what later work should keep, and
the person keeps or drops each draft. Add it as one ask, as the card says, and write nothing before the
person says yes.

1. Read the source. For past chats, read the agent's history files the person names, such as
   `~/.claude/projects/<project>/*.jsonl`, and look at the person's own prompts: corrections, the
   words "always" and "never", picks, and reactions to a result. For a skill file, a folder or a URL,
   read its text and the files it shows as examples.
2. Run `celis search -- "<what the source is about>"`, and leave out each draft that a live rule or
   pattern already says.
3. Draft each point as a rule or a pattern, as the card's Learn move says: a must, such as "always" or
   "never", is a rule; a taste, shown by a result the person liked or did not, is a pattern, with each
   file, step or URL the source gives as a good or a bad example. Write each draft as "When <what you
   see>: <what to do>, because <the reason in the source>." Name a secret by where it lives, never by
   its value.
4. Show the person the drafts, numbered, each with where in the source it came from, and ask which to
   keep: a list of numbers, all, or none. Write each kept draft with `decide`, its `--path` as narrow
   as it allows, and `--scope me` when it holds for that kind of work in any project.
5. Close the ask. In the end, give how many drafts you made and how many the person kept, and each id.
