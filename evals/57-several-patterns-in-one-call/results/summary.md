# Eval 57: read several patterns in one call

The case asks for a check in `order.py` and a unittest test. Three of the four patterns in its store
bear on the task. It compares the card and the patterns agent at `56630bc` (old) with those that show
`celis patterns <id>...` (new). Each arm ran 5 times on claude-sonnet-5-5, with the patterns agent on
Haiku and the celis built from the crates at `3173c01`, on 02-11-2026.

| Grader                   | Old card | New card |
| ------------------------ | -------- | -------- |
| several-ids-in-one-call  | 0/5      | 5/5      |
| no-chained-reads         | 1/5      | 5/5      |
| named-uses               | 5/5      | 5/5      |
| tests-pass               | 5/5      | 5/5      |
| Cost                     | $0.53    | $0.53    |

With the old card, the patterns agent read the patterns one id at a time: 4 runs chained them as
`celis patterns A; echo ---; celis patterns B`, and 1 run made a call for each. With the new card, each
run read all four in one `celis patterns A B C D` call. The case does not measure `celis tree`.
