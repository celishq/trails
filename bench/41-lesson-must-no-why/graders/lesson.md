---
type: llm
focus: trace
arm: with-only
---
Read each `celis pattern` command in the trace and the final reply. Pass only if all of these hold:

- The patterns hold only the lesson to use pathlib and never os.path: no pattern about list_logs, .log files or sorting.
- Each pattern has four lines, When, Forces, Do and Because, and its When names the kind of work, never "this project", the repo or the person.
- Each Because gives a reason the choice is good for the work, such as what goes wrong without it; a Because that says only that the person asked, wants or prefers it fails.
- The person gave no reason, and the final reply names the Because of the pathlib pattern as a guess for the person to confirm, such as a "guessed" line or a Learned line that says the why is guessed.
