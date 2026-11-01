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
