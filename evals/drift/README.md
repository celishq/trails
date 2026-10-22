# The drift bench

Does an agent keep the rules it has been given as its memory grows? This bench compares a CLAUDE.md that
holds every rule with Celis, from 40 to 1,600 rules.

## How it works

We took 1,600 coding rules from the agent instruction files of 13 large open-source repos: VS Code, Next.js,
PostHog, Airflow, Grafana, ruff, LangChain, n8n, Kiali, kube-router, the OpenAI Agents SDK, the MariaDB
operator and React on Rails. We rewrote them for one small Python app, `seed/`. Each rule in
`data/filler.json` names the repo it came from.

The agent does 8 everyday tasks on that app (`data/tasks.json`), like a CSV export, a backup or a SQLite
store. After each task, `check.py` checks 34 of the rules in the code it wrote (`data/targets.json`). 4 of
those rules changed over time: an older version says one thing, and a newer one replaces it.

Each task starts from a new copy of the app and of the memory, so tasks don't affect each other. The setups:

- **CLAUDE.md** (`oracle`): every rule in a CLAUDE.md the agent reads at the start. Like a real file, it
  still holds the old version of each rule that changed, above the new one.
- **Celis** (`final`): the same rules in Celis, each with the files it is about. A new version replaces the
  old one.
- **No memory** (`plain`): neither.

Rules that the agent keeps even with no memory are habits, not memory, so `report.py` leaves them out.

## Results

Claude Code with Opus, 8 tasks at each size:

| Rules in memory | Outdated rules followed, CLAUDE.md | Outdated rules followed, Celis | Rules broken, CLAUDE.md | Rules broken, Celis | Cost per task, CLAUDE.md | Cost per task, Celis |
| --------------- | ---------------------------------- | ------------------------------ | ----------------------- | ------------------- | ------------------------ | -------------------- |
| 40              | 61%                                | **0%**                         | 29%                     | **14%**             | $0.31                    | $0.43                |
| 100             | 71%                                | **11%**                        | 28%                     | **24%**             | $0.35                    | $0.49                |
| 400             | 83%                                | **23%**                        | 37%                     | **31%**             | $0.55                    | $0.61                |
| 1,000           | 79%                                | **41%**                        | **48%**                 | 51%                 | $0.84                    | **$0.66**            |
| 1,600           | 100%                               | **52%**                        | 70%                     | **57%**             | $1.04                    | **$0.73**            |

With no memory, the agent broke 95% of the rules.

![Outdated rules the agent followed](../../media/outdated-rules.svg)

![Rules the agent broke](../../media/rules-broken.svg)

## How many rules a real repo has

The marks under each chart count the bullet points in the root agent file of some of the source repos, as
of October 2026. Most of them have more agent files in their folders.

| Repo                                                          | Rules in its root agent file |
| ------------------------------------------------------------- | ---------------------------- |
| [VS Code](https://github.com/microsoft/vscode)                | 79                           |
| [LangChain](https://github.com/langchain-ai/langchain)        | 94                           |
| [Airflow](https://github.com/apache/airflow)                  | 110                          |
| [Next.js](https://github.com/vercel/next.js)                  | 123                          |
| [PostHog](https://github.com/PostHog/posthog)                 | 160                          |
| [Kiali](https://github.com/kiali/kiali)                       | 229                          |
| [kube-router](https://github.com/cloudnativelabs/kube-router) | 282                          |

## Limits

- Each size is 8 tasks. Celis ran twice and CLAUDE.md once. Two runs of the same setup differ by up to 15
  points, so treat the small gaps as a tie.
- Only 4 of the checked rules changed over time, so each "outdated rules" point rests on a few dozen checks.
- All the rules are conventions for one Python app. Other kinds of rules may behave differently.

## Run it

```bash
python3 run.py final 40 100 400 1000 1600 --model opus      # Celis; CELIS_BIN and CELIS_PLUGIN pick the build
python3 run.py final 40 100 400 1000 1600 --model opus --again   # a second run, added to the first
python3 run.py oracle 40 100 400 1000 1600 --model opus     # CLAUDE.md
python3 run.py plain 0 --model opus                         # no memory
python3 report.py opus                                      # the table of every setup
python3 chart.py                                            # redraws the two charts in media/
```

Runs live in `$DRIFT_RUNS` (default `/tmp/celis-drift`). Each task adds one line to
`results/<setup>-<model>-r<rules>.jsonl`.
