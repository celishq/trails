# Eval 53: offer /celis:learn on a first session with no pattern

The case asks for a negative-age check in `age.py` with an empty store. Each arm ran 5 times on
claude-sonnet-5-5, on 31-10-2026.

| Grader       | Offer "after the first ask" | Offer "at the end of the answer to the first ask" |
| ------------ | --------------------------- | ------------------------------------------------- |
| offers-learn | 2/5                         | 5/5                                               |
| did-the-work | 5/5                         | 5/5                                               |
| Cost         | $0.49                       | $0.62                                             |

With "after the first ask", 3 runs left the offer for a later answer that a one-prompt run never gives.

## The status line

The grader reads the trace for a first message of the agent that starts with the status line. An LLM
grader on the final answer missed it, because the agent gives it in its first message, before the work.
Counts are of case 53 and of the removed case below, 5 runs each, rescored from the kept traces.

| Wording                                                             | Status line |
| ------------------------------------------------------------------- | ----------- |
| Old card                                                            | 0/5 (removed case) |
| Card: "In your first answer of a session, give the person ... line" | 1/10        |
| Session-start line: "Give the person this line in your first answer" | 5/10        |
| Card: "Start your first answer ... word for word, as its first line: the person does not see what the session start gives you ..." | 10/10 |
| The same rule at the top of the card                                 | 10/10       |

## Naming the pattern

A removed case asked for a negative-age check with a `me` pattern on error messages. The old card and a
card that asked the agent to name the pattern that changed a choice both named it in 5 of 5 runs, so the
card keeps no line for it. Case 08 scored 5/5 on every grader with both cards.
