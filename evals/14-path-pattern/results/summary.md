# Evals 14 and 29: a path as the module where a pattern holds

Before: the build at `dba4218`, where `step done` lists the patterns of changed files and the brief asks
for a path to the files a pattern acts on. After: the build at `ae7a1ba`, where `done` lists none and a
path is the module or kind of file where a pattern holds, read from the index. Each arm ran 3 times on
Opus, with no baseline arm, on 29-10-2026.

| Case                  | Before        | After         |
| --------------------- | ------------- | ------------- |
| 14-path-pattern       | 3/3, 9 turns  | 3/3, 6-8 turns |
| 29-pattern-at-choice  | 3/3, 9-10 turns | 3/3, 9-12 turns |
| Cost                  | $1.15         | $1.12         |

In both arms of eval 14, each run named the shop pattern by its id and logged the price change with
`log.info`. With no list at `done`, the agent found it in the index alone.
