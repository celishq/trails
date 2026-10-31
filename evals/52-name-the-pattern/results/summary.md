# Eval 52: name the pattern, and give the status line

The case asks for a negative-age check in `age.py`, with a `me` pattern on error messages. It compares
the card at `ae31e05` (old) with the card that asks the agent to name a pattern that changed a choice
and to give the status line (new). Each arm ran 5 times on claude-sonnet-5-5, on 31-10-2026.

| Grader            | Old card | New card, rule in the card | New card, rule in the status line |
| ----------------- | -------- | -------------------------- | --------------------------------- |
| message-like-good | 5/5      | 5/5                        | 5/5                               |
| names-the-pattern | 5/5      | 5/5                        | 5/5                               |
| Cost              | $0.73    | $0.76                      | $0.69                             |

The old card already names the pattern in 5 of 5 runs, so the card keeps no line for it. Case 08 scored
5/5 on every grader with both cards. Case 29 finds no pattern in this harness, because its setup names
another project than the run.

## The status line

The grader reads the trace for a first message of the agent that starts with the status line. An LLM
grader on the final answer missed it, because the agent gives it in its first message, before the work.
Counts are of cases 52 and 53, 5 runs each, rescored from the kept traces.

| Wording                                                             | Status line |
| ------------------------------------------------------------------- | ----------- |
| Old card                                                            | 0/5 (52)    |
| Card: "In your first answer of a session, give the person ... line" | 1/10        |
| Session-start line: "Give the person this line in your first answer" | 5/10        |
| Card: "Start your first answer ... word for word, as its first line: the person does not see what the session start gives you ..." | 10/10 |
| The same rule at the top of the card                                 | 10/10       |
