# Eval 48: start each step, and keep source out of proof files

The case asks for a coupon in `cart.py` and a unittest test. It compares the card at `fe25c45` (old)
with the card that says to start each step and what a proof file is (new). Each arm ran 5 times on
claude-sonnet-5-5, with the celis built from the crates at `d755c5a`, on 29-10-2026.

| Grader                | Old card | New card |
| --------------------- | -------- | -------- |
| no-source-proof-file  | 0/5      | 5/5      |
| started-before-work   | 5/5      | 5/5      |
| step-done             | 5/5      | 5/5      |
| tests-pass            | 5/5      | 5/5      |
| Cost                  | $0.40    | $0.41    |

With the old card, each run gave `cart.py` or `test_cart.py` as a `--file` of its proof. With the new
card, no run did. Both cards started the step before the work in this case, so it does not show that
the start line changes what an agent does.

## The end line

On 02-11-2026, the card that says to end the answer with the last line of the ask's `done` ran 5 times on
claude-sonnet-5-5, with the celis built from the crates at `7bdd5f58`, for $0.51. `end-line` passed in
5 of 5 runs, and the other graders kept 5 of 5. No run of the old card was made: it has no rule for the
line, and the old `done` printed no such line.
