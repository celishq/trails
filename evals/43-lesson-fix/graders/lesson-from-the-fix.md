---
type: llm
focus: trace
arm: with-only
---
The fix changed created_at from the server's local time with no offset to UTC with its +00:00 offset, after the Berlin warehouse saw orders an hour off. Pass only if a written pattern says that timestamps written for another system are UTC, or carry their offset, and not the local time of the server.
