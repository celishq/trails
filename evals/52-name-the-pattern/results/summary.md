# Eval 52: name the pattern, and give the status line

The case asks for a negative-age check in `age.py`, with a `me` pattern on error messages. It compares
the card at `ae31e05` (old) with the card that asks the agent to name a pattern that changed a choice
and to give the status line (new). Each arm ran 5 times on claude-sonnet-5-5, on 31-10-2026.

| Grader            | Old card | New card, rule in the card | New card, rule in the status line |
| ----------------- | -------- | -------------------------- | --------------------------------- |
| message-like-good | 5/5      | 5/5                        | 5/5                               |
| names-the-pattern | 5/5      | 5/5                        | 5/5                               |
| status-line       | 0/5      | 1/5                        | 0/5                               |
| Cost              | $0.73    | $0.76                      | $0.69                             |

The old card already names the pattern in 5 of 5 runs, so the card keeps no line for it. The status
line rule in the card gave the line in 3 of 10 runs of cases 52 and 53, and the same rule in the hook's
status line in 0 of 10, so the rule stays in the card. Case 08 scored 5/5 on every grader with both
cards. Case 29 finds no pattern in this harness, because its setup names another project than the run.
