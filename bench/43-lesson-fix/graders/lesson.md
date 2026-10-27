---
type: llm
focus: trace
arm: with-only
---
Read each `celis pattern` command in the trace and the final reply. Pass only if all of these hold:

- The fix changed created_at from the server's local time with no offset to UTC with its +00:00 offset, after the Berlin warehouse saw orders an hour off; a pattern says that timestamps written for another system are UTC, or carry their offset, and not the local time of the server.
- The patterns hold only the lesson about time zones of timestamps: no pattern about the Berlin warehouse alone, the created_at field name or export.py alone.
- Each pattern has four lines, When, Forces, Do and Because, and its When names the kind of work, never "this project", the repo or the person.
- Each Because gives a reason the choice is good for the work, such as what goes wrong without it; a Because that says only that the person asked, wants or prefers it fails.
