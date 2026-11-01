---
type: llm
focus: trace
arm: with-only
---
Read each `celis pattern` command in the trace and the final reply. Pass only if all of these hold:

- The person picked version B (msg-b.txt, from the step "Write version B of the message in msg-b.txt") over version A (msg-a.txt); the pattern for the error message gives `--good` with msg-b.txt or the id of the step of B, and `--bad` with msg-a.txt or the id of the step of A.
- B said what failed and what to do next in two short sentences, and A was chatty and vague; the Do says to keep an error message short and plain, saying what failed and what to do, and not chatty.
- Each pattern has four lines, When, Forces, Do and Because, and its When names the kind of work, never "this project", the repo or the person.
- Each Because gives a reason the choice is good for the work, such as what goes wrong without it; a Because that says only that the person asked, wants or prefers it fails.
