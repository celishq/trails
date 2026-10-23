# Demo script: a rule comes right before an edit

A 45-second screen recording. The left pane is Claude Code, the right pane is `celis tree`. The project is
the shop from the README example, with a rule from last week: "set prices in prices.py, not the cart".

| Time      | Screen                                                                                    | Caption                                   |
| --------- | ----------------------------------------------------------------------------------------- | ----------------------------------------- |
| 0:00–0:03 | Title card.                                                                               | Stop babysitting your coding agent.       |
| 0:03–0:05 | The person types: "Students pay 10% less."                                                | —                                         |
| 0:05–0:09 | The agent opens `cart.py` to edit it. Before the edit, `celis rules cart.py` prints the rule. Zoom on the rule. | You said this once, last week.            |
| 0:09–0:15 | The agent closes `cart.py` and edits `prices.py`.                                         | It changed course before the mistake.     |
| 0:15–0:22 | Cut to last week: the person writes "No, prices live in prices.py, not the cart." The agent runs `celis decide`. | One correction became a rule.             |
| 0:22–0:30 | Back to today. Before done, `celis rules --diff` prints "tests need SHELF_OFFLINE=1". The test passes with it. | And another one, right before it's done.  |
| 0:30–0:38 | The right pane: the trail of the job, with each step and its proof.                       | See what it did and why.                  |
| 0:38–0:45 | End card: the headline, the subtitle and the install command.                             | `curl -fsSL https://celis.ai/install.sh \| sh` |

## Recording notes

- Record a real session, not a mock. Write the two rules last week's way, through a correction, so the
  cut at 0:15 is real.
- Speed up the agent's thinking; keep each rule on screen for at least 2 seconds.
- Put the subtitle on the end card when it is chosen.
