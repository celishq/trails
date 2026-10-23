# Pain research

Phase 2 of `plans/23-10-2026-positioning.md`.

## Search queries

Source: Similarweb keyword generator, Google, worldwide, the 28 days to 2026-10-20. Volume is searches in
those 28 days.

| #   | Query                                          | Volume | Pain or intent                       |
| --- | ---------------------------------------------- | ------ | ------------------------------------ |
| 1   | claude-mem                                     | 25.7K  | Looks for a memory plugin by name    |
| 2   | agents.md                                      | 10.3K  | Writes a rule file                   |
| 3   | claude.md                                      | 9.8K   | Writes a rule file                   |
| 4   | agent memory                                   | 6.8K   | Category search                      |
| 5   | claude-mem github                              | 6.7K   | Looks for a memory plugin by name    |
| 6   | cursor rules                                   | 5.0K   | Writes a rule file                   |
| 7   | claude code hooks                              | 4.7K   | Wants the agent to act at a moment   |
| 8   | /compact claude code                           | 4.6K   | Loses context in long sessions       |
| 9   | claude code best practices                     | 4.6K   | Wants better results                 |
| 10  | claude code memory                             | 3.2K   | Wants the agent to remember          |
| 11  | claude code rules                              | 2.3K   | Writes rules                         |
| 12  | claude.md vs agents.md                         | 2.0K   | Keeps rules for more than one agent  |
| 13  | codex cursor skill for infinite memory         | 2.0K   | Wants memory across agents           |
| 14  | codex memory                                   | 1.6K   | Wants the agent to remember          |
| 15  | how do i keep one claude codes session memory separate from other ones | 1.1K | Memory leaks between sessions     |
| 16  | what's better than claude-mem                  | 760    | Compares memory plugins              |
| 17  | memsearch vs claude-mem vs mem0                | 680    | Compares memory plugins              |
| 18  | why is my claude code project not having any memory of what it did earlier | 670 | Forgets past work |
| 19  | how to give claude code persistent memory      | 540    | Wants the agent to remember          |
| 20  | cursor sucks, it constantly drifts form the rules | 500 | Breaks rules                          |
| 21  | claude ignores claude.md                       | 102    | Breaks rules                         |

## Findings

1. People search for the solution ("memory", "claude-mem"), not for the pain ("ignores", "forgets").
   Queries with "ignores" have about 100 searches. Queries with "memory" have thousands.
2. claude-mem owns the category search: 25.7K, more than "agent memory" itself. Comparison queries
   (rows 16, 17) are the easiest place to enter.
3. Rule files are a large search: agents.md, claude.md and cursor rules add up to about 25K. A reader
   who writes a rule file already has the problem that Celis solves when the file grows.
4. Cross-agent queries exist (rows 12, 13): people want one set of rules for Claude, Codex and Cursor.
5. "Discipline" and "babysitting" have no search volume. They work for the pitch, not for search pages.

## Search pages to write (phase 3)

1. Celis compared to claude-mem and mem0 (rows 1, 16, 17).
2. Why Claude Code ignores your CLAUDE.md, and how to fix it (rows 3, 9, 21).
3. One set of rules for Claude Code, Codex and Cursor (rows 2, 12, 13).
4. Claude Code memory that lasts across sessions and projects (rows 10, 18, 19).
5. Rules that survive /compact (row 8).
