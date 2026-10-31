# Eval 53: offer /celis:learn on a first session with no pattern

The case asks for a negative-age check in `age.py` with an empty store. Each arm ran 5 times on
claude-sonnet-5-5, on 31-10-2026.

| Grader       | Offer "after the first ask" | Offer "at the end of the answer to the first ask" |
| ------------ | --------------------------- | ------------------------------------------------- |
| offers-learn | 2/5                         | 5/5                                               |
| did-the-work | 5/5                         | 5/5                                               |
| tests-pass   | 5/5                         | 5/5                                               |
| Cost         | $0.49                       | $0.62                                             |

With "after the first ask", 3 runs left the offer for a later answer that a one-prompt run never gives.
