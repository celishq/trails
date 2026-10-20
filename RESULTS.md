# Benches

Two benches ask one question: when a reviewer tells an agent how the work must be done, does the agent keep it
in the next session, without being told again?

- **Raycast:** the rules must hold in other projects. The bench code is in `evals/raycast/`.
- **Landing:** the rules must hold in the same project, and from Claude Code to Codex. The bench code is in
  [celishq/benches](https://github.com/celishq/benches), in `landing/`.

## The Raycast bench: does a review hold in another project?

An agent does 10 real Raycast extension PRs, one after another, each in a different extension. After the first
two, a reviewer gives 5 rules. We count how often the next 8 tasks break them, and how many review rounds it
takes to fix them.

**Result (1 run per arm):**

- Sonnet broke the rules 6 times without celis and 3 times with it.
- Haiku broke them 16 times without celis and 3 times with it, at a quarter of Sonnet's cost.
- Opus with Haiku subagents broke them 6 times without celis and once with it.

Every arm built every extension.

### Why this bench

A rule in a project's `CLAUDE.md` holds in that project. This bench puts each task in a new folder that holds
only its own extension, so no project file carries a rule from one task to the next. The rule reaches a later
task only through the arm's memory.

### Tasks

The tasks are 10 merged PRs of [raycast/extensions](https://github.com/raycast/extensions), all opened after the
model's training data ends. Each PR changes one extension.

| # | PR | Extension | Work |
|---|---|---|---|
| 1 | #31771 | linkpilot | New extension |
| 2 | #30017 | bitbucket | Update |
| 3 | #30395 | migadu | Fetch domains through the API |
| 4 | #30336 | digikey | New extension: search for products |
| 5 | #31801 | tapo-smart-devices | Faster loading and instant on/off |
| 6 | #30016 | youtrack | Live search and autocomplete |
| 7 | #31207 | linqlo | New extension |
| 8 | #31850 | aktar | Watched folders |
| 9 | #30215 | zoho-mail | New extension |
| 10 | #31374 | dokploy | Deploy Service command |

Each task starts as a new git repo with the extension at the PR's base commit (empty for a new extension), and
`npm install`. The prompt is the PR's title and description, without the screencast, the checklist and the lines
that report the author's tests.

### The review

After task 1 and task 2, a new session gets the review:

```
Review feedback on the `<extension>` extension: <review> <review> Fix it. Go on to the end without asking.
```

| Rule | After task | Review | Tasks that test it |
|---|---|---|---|
| changelog | 1 | A new CHANGELOG.md entry whose heading ends with `{PR_MERGE_DATE}`, not a date. | 2 to 10 |
| failure-toast | 1 | Errors use `showFailureToast`, not `showToast` with `Toast.Style.Failure`. | 2 to 10 |
| console | 1 | No `console.log` left in the source. | 2 to 10 |
| data-hooks | 2 | Data loads with `useFetch`, `usePromise` or `useCachedPromise`, not `useEffect` with `useState`. | 3 to 10 |
| no-tests | 2 | No test files and no `test` script. | 3 to 10 |

These are real Raycast review conventions. Together they give 43 rule-tasks per run (3 rules × 9 tasks, plus
2 rules × 8 tasks). A script checks each rule on the lines the task adds, so no model grades the work. The
**gold** arm (the merged code) breaks 13 of 43: the changelog on all 9 tasks (the Raycast bot fills in the
placeholder at merge, so the merged code always has a date), failure-toast on 2 and no-tests on 2.

After each task, while a given rule is broken, a new session gets the same review again, for the broken rules
only, up to 3 times. Each check also runs `ray build` and `ray lint`.

### Arms

Every session is a new `claude -p` process (Claude Code 2.1.304), with the user's settings, plugins and MCP
servers left out:

```
claude -p "<prompt>" --model <model> --output-format stream-json --verbose \
  --dangerously-skip-permissions --setting-sources project --strict-mcp-config \
  --max-budget-usd 5 --no-session-persistence
```

| Arm | What it adds |
|---|---|
| **plain** | Nothing. |
| **celis** | The celis plugin (`--plugin-dir`) and `celis` on `PATH`, from this repo at `db82b7d`. Each run has its own empty store. |

Each arm runs on three model setups:

| Setup | Models |
|---|---|
| Sonnet | `claude-sonnet-5-5` |
| Haiku | `claude-haiku-5-5` |
| Team | `claude-opus-5-5` plans and gives every edit and command to a `worker` subagent on `claude-haiku-5-5` |

### Results (1 run per arm)

| Setup | Arm | Broken, of 43 | Review rounds | Left broken | Cost | Agent minutes |
|---|---|---|---|---|---|---|
| Sonnet | plain | 6 | 6 | 0 | $2.60 | 18 |
| Sonnet | **celis** | **3** | **3** | 0 | $2.70 | 22 |
| Haiku | plain | 16 | 17 | 3 | $0.70 | 43 |
| Haiku | **celis** | **3** | **7** | 2 | $0.60 | 39 |
| Team | plain | 6 | 5 | 0 | $3.10 | 54 |
| Team | **celis** | **1** | **1** | 0 | $4.60 | 72 |

Rules broken, by rule:

| Setup | Arm | changelog | failure-toast | console | data-hooks | no-tests |
|---|---|---|---|---|---|---|
| Sonnet | plain | 2 of 9 | 1 of 9 | 0 of 9 | 1 of 8 | 2 of 8 |
| Sonnet | celis | 0 | 1 | 0 | 0 | 2 |
| Haiku | plain | 9 | 3 | 0 | 1 | 3 |
| Haiku | celis | 2 | 0 | 0 | 0 | 1 |
| Team | plain | 1 | 2 | 0 | 1 | 2 |
| Team | celis | 0 | 0 | 0 | 0 | 1 |

### How to read it

- **Haiku gains the most.** Without celis, Haiku used a fixed date in all 9 changelogs. With celis, it broke
  that rule twice, both times in a new extension. Haiku with celis kept the rules as well as Sonnet with celis,
  at a quarter of the cost.
- **A team needs the rules at each handoff.** Opus without celis broke 1 rule in tasks 1 to 5, then 3 on task 6
  and 2 more on tasks 7 and 8. With celis, the planner gave each worker a celis step, and the worker's
  `celis step start` printed the rules for it.
- **celis does not always save money.** For Sonnet and Haiku, the cost is about the same with and without
  celis. For the team, celis costs 48% more, because the Opus planner runs `celis:past` and hands out steps.
- console never broke in any arm, so it does not separate the arms.

### Limits

- 1 run per arm, on the same 10 tasks. Sonnet without celis broke 14 rules in an earlier pilot, with an older
  version of the check, so one run says little about the spread.
- celis was tuned on these tasks. Its card and prompt hook changed three times after we looked at the failures
  here: review rules now become rules even in an empty store, a rule about a platform gets the `me` scope, and a
  rule's "When" names the work, not the review. The next step is 10 new PRs that celis was not tuned on, with
  5 runs per arm.
- There is no arm with the rules in a user-level `CLAUDE.md` for every setup yet. On Sonnet, that arm is built
  (`oracle`) but has not run on this version of the bench.
- 5 rules and 10 tasks. The bench does not check how good the feature is, only whether the rules hold.

### Run it

You need macOS or Linux, Node 22 or later, Python 3, `gh` signed in to GitHub, and Claude Code.

```bash
cd evals/raycast
git clone --filter=blob:none --no-checkout https://github.com/raycast/extensions cache/extensions
python3 tasks.py                        # writes data/tasks.json and downloads each task's files
python3 run.py gold 1
export CELIS_BIN=<folder with celis> CELIS_PLUGIN=<this repo>/plugins/celis
for model in sonnet haiku team; do
  for arm in plain celis; do python3 run.py $arm 1 --model $model; done
done
python3 report.py
```

A run of one arm takes 20 to 70 minutes and $0.60 to $5.

## The landing bench: does an agent keep a code review?

An agent builds 8 real pages of the Supabase marketing site, one after another. After the first page, a
reviewer gives two rules. We count how often the agent breaks those rules on the next 7 pages, and what it
costs to fix them. We compare plain Claude Code, Claude Code with a `CLAUDE.md`, and Claude Code with celis.

**Result (1 run per arm):** without memory, the agent broke the review's rules 22 times and needed 5 more
review rounds. With a `CLAUDE.md`, the result was the same. With celis, it broke them 0 times and needed no
more rounds, at 13% less cost and 20% fewer input tokens.

**Cross-agent (1 run per arm):** when Claude Code hears the review and Codex does the next 7 pages, Codex broke
the rules 19 times without celis and 2 times with it.

The bench code is in [celishq/benches](https://github.com/celishq/benches), in `landing/`.

### The question

A reviewer tells an agent how the team works: "ship images as webp", "keep descriptions short". The agent fixes
the page. Does the agent keep the rule on the next task, in a new session, without being told again?

### The experiment

#### Tasks

The tasks are 8 merged pull requests of [supabase/supabase](https://github.com/supabase/supabase) that change
pages of `apps/www`, the Supabase marketing site. Each PR:

- was opened on or after 1 Jul 2026, after the training data of the model ends (Jun 2026);
- was written by a person, not a bot;
- adds or rebuilds a page that renders on the dev server without production secrets.

| # | PR | Base commit | Merge commit | Pages | Work |
|---|---|---|---|---|---|
| 1 | #47750 | `26248be75` | `0675075a3` | `/customers/delightai`, `/customers/drew-crew`, `/customers/lovable` | 3 customer stories, 11 images |
| 2 | #47753 | `72aa214b0` | `949a57d28` | `/company` | Rebuild the investor logo section |
| 3 | #48798 | `66a4a0b32` | `6ac031673` | `/customers/qa-tech` | Customer story |
| 4 | #49595 | `e8f9359a9` | `8291ed140` | `/customers/lingo-dev` | Customer story with quote portraits |
| 5 | #49306 | `b66258c0b` | `292c08b7b` | `/regions` | New page: map, list, 17 flags |
| 6 | #47874 | `0e6fff676` | `51a167d91` | `/partners` | New page with sections and a form |
| 7 | #50899 | `1f5d10e69` | `4617b6f4c` | `/compute` | New product page with an animation |
| 8 | #51285 | `09a245d72` | `47fd296fc` | `/customers/flick-games` | Customer story |

The tasks run in merge order, as one chain. Each task starts from the base commit of its PR, so code never
carries over from one task to the next. Only the arm's memory carries over.

#### What the agent gets

For each task, the harness puts these files in `.bench/<pr>/` in the repo:

- a full-page screenshot of each merged page at 1440 px and at 375 px wide;
- the visible text of each merged page;
- the images that the PR adds, as the PR adds them (png, jpg, svg).

The agent does not get the merged code. The prompt is:

```
Build or update these pages of the Supabase marketing site, apps/www, from the design in .bench/<pr>/:
- <route>: frames <name>-desktop.png (1440 px wide) and <name>-phone.png (375 px), text <name>.txt
New images are in images/. Match the frames. If you start the dev server, use port <port>.
Go on to the end without asking.
```

#### The review

After a task that a rule comes from, a new session gets the review:

```
Review feedback on <routes>: <rule> <rule> Fix the pages. Go on to the end without asking.
```

| Rule | After task | Text | Tasks that test it |
|---|---|---|---|
| webp | 1 | Raster images ship as .webp. | 2 to 8 |
| description | 1 | A meta description has at most 160 characters. | 2 to 8 |
| alt | 4 | An image next to a visible name is decorative: its alt is empty. | 5 to 8 |
| reduced-motion | 5 | Every animation respects prefers-reduced-motion: with reduced motion, nothing loops or pulses. | 6 to 8 |

We chose the webp and description rules so that a fault is the easy path. Nothing in the code, the screenshots
or the tooling enforces them. The repo breaks both: it ships png and jpg images, and the customer stories next
to the new ones have descriptions of 185 to 272 characters. The merged PRs break them too. Neither rule
changes how a page looks, so the screenshots do not push the agent either way.

The alt and reduced-motion rules come from the real reviews of tasks 4 and 5. No arm broke them, so they do not
separate the arms.

#### Follow-ups

After each task, while a rule from an earlier review is broken, a new session gets the same review again, for
the broken rules only. The limit is 3 follow-ups a task. This gives two results:

1. **Single shot:** the faults after the first session of each task.
2. **To clean:** the review rounds, minutes, cost and tokens until every rule holds.

#### Arms

Every session is a new `claude -p` process in the working copy of the arm:

```
claude -p "<prompt>" --model claude-sonnet-5-5 --output-format stream-json --verbose \
  --dangerously-skip-permissions --setting-sources project --strict-mcp-config \
  --max-budget-usd 5 --no-session-persistence
```

`--setting-sources project` and `--strict-mcp-config` keep the user's own settings, plugins and MCP servers out
of every arm. The working copies are in `/tmp`, so no `CLAUDE.md` from a parent folder reaches the agent. The
Supabase repo's own `.claude/` (its `CLAUDE.md`, skills, and a hook that runs Prettier and ESLint) stays in every
arm, because it is part of the real project.

| Arm | What it adds |
|---|---|
| **plain** | Nothing. |
| **static** | A `CLAUDE.md` at the repo root with 4 lines of good working habits (below). It does not change during the run. |
| **celis** | The celis plugin (`--plugin-dir`) and the `celis` command on `PATH`. Each run has its own empty store (`CELIS_HOME`). |

The static `CLAUDE.md`:

```
# How to work

- Start with the big picture. Find the parts of the work, and take first the part you are least sure of.
- Take small steps, one at a time. Plan only the next step, then do it.
- Show proof for each step: a command and its output, or a screenshot you compared with the design.
- When a step fails, takes too long or must be redone, say why before you go on.
```

celis ran from [celishq/trails](https://github.com/celishq/trails) at `b8d4be2` with one change to the prompt
hook (see "How celis got there"). The change will be in the next release.

#### Measures

After each session, the harness runs `content:build` and the Next.js dev server of `apps/www`, and checks each
page of the task in headless Chromium:

| Measure | How |
|---|---|
| Shown | The page returns 200. A failed content build counts as not shown. |
| Likeness | 1 − the share of content pixels that differ from the merged page (pixelmatch, threshold 0.1), at 1440 px and 375 px. Content pixels are the pixels off the page background in either image. Animations are off. |
| Text recall | The share of the merged page's text lines that the page shows. |
| webp faults | The png and jpg files that the task adds under `apps/www/public`. |
| description faults | Pages whose `og:description` (else `meta[name=description]`) has more than 160 characters. |
| alt faults | Images outside the header, nav and footer whose alt repeats the text next to them. |
| reduced-motion faults | Endless animations that still run 1 s after load with reduced motion. |
| axe | Serious and critical axe-core violations. |
| Cost, minutes, tokens | From the `result` event of each session's stream-json. |

A rule counts on a task only when its review came before that task. Two control arms check the measures: **gold**
(the merged code) shows every page with likeness 1.00 and 0 rule faults, and **base** (no change) shows 2 of
8 tasks, with likeness 0.50.

#### Environment

| | |
|---|---|
| Machine | Apple M1 Pro, 16 GB, macOS 15.7.9 |
| Claude Code | 2.1.300 |
| Codex CLI | 0.162.1, model `gpt-5.6-terra`, medium reasoning |
| Model | `claude-sonnet-5-5` |
| Node | 26.5.0, with pnpm 10.24.0 for the Supabase monorepo |
| Python | 3.14.0 |
| Checks | playwright 1.64.0, pixelmatch 6.0.0, pngjs 7.0.0, @axe-core/playwright 4.10.1 |
| Date | 15 Oct 2026 |

### Results

#### Result 1: single shot

Tasks 2 to 8, after the review in task 1.

| | plain | static | **celis** |
|---|---|---|---|
| Review rules broken | 22 | 22 | **0** |
| png and jpg files shipped | 18 | 18 | **0** |
| Descriptions over 160 characters | 4 | 4 | **0** |
| Tasks with a fault | 4 of 7 | 4 of 7 | **0 of 7** |
| Tasks that show all pages (tasks 1 to 8) | 7 of 8 | 8 of 8 | 8 of 8 |
| Desktop likeness | 0.83 | 0.85 | 0.78 |
| Phone likeness | 0.80 | 0.83 | 0.77 |
| Text recall | 0.97 | 0.98 | 0.99 |

#### Result 2: to clean

All 8 tasks, until each rule holds.

| | plain | static | **celis** |
|---|---|---|---|
| Follow-up reviews | 5 | 5 | **0** |
| Rules still broken at the end | 0 | 0 | 0 |
| Cost of the follow-ups | $0.42 | $0.49 | **$0** |
| Agent minutes in the follow-ups | 1.7 | 3.2 | **0** |
| Total cost | $2.87 | $3.18 | **$2.49** |
| Total agent minutes | 21.6 | 21.0 | 20.6 |
| Input tokens, with cache | 5.96M | 7.12M | **4.76M** |
| Output tokens | 79k | 93k | **70k** |
| Desktop likeness after the fixes | 0.70 | 0.76 | 0.78 |

celis costs 13% less than plain and 22% less than static, and uses 20% fewer input tokens than plain. The agent
minutes are almost the same. Each follow-up also adds a rebuild and check of about 2 minutes, and a human round
trip that the bench does not measure.

#### How to read it

- The reviewer gave the feedback once. Without celis, the reviewer had to give it again on 4 of the next 7
  tasks. A static `CLAUDE.md` did not help, because the rules came up during the work and never reached the file.
- The money and token savings are real but small. A follow-up is cheap: about $0.08 and 20 s of agent time.
  The clearer number is the review rounds: 5, 5 and 0.
- Page likeness is about the same in all arms. The 3 new layout pages (`/regions`, `/partners`, `/compute`)
  are 27% to 50% off in every arm.
- celis's lower likeness comes from task 1. In the review session it shortened the description that the page
  also shows (46% off). The follow-ups in plain did the same, and its likeness fell from 0.83 to 0.70.
- plain broke the content build in its task 1 review, so its task 1 shows no pages. celis did the same in an
  earlier pilot, so this is an agent risk, not an effect of the arm.

#### Why celis made 0 faults

Each later session ran the `celis:past` agent first, because the prompt hook asks for it. In all 7 sessions,
`celis past` gave the review rules in full, at the top of its output:

```
01d57b7c project When writing a meta description: keep it at most 160 characters, because review feedback required it
7b5e4044 me When adding raster images to pages: ship them as .webp, because review feedback required it
```

The agent then acted on them. It ran `cwebp` on each task with raster images (tasks 2, 3, 4 and 8), and it gave
each page a short `meta_description`. It also shortened the existing `/company` description (172
characters). The rules are short and mechanical, so once the agent sees one, it keeps it.

0 is not the expected rate. The same celis build made 3 faults in the run before this one: it left the
`/company` description, and it shipped 2 png files on task 4. With 1 run per arm, expect about 0 to 3 faults
over 7 tasks.

The agent wrote the webp rule with the `me` scope, so it holds in every project of that store. In a real store,
the rule would reach other projects. The bench cannot see this, because each run has its own store.

### Cross-agent: say it to Claude Code, Codex keeps it

The same bench, with two agents. Claude Code (Sonnet 5.5) does task 1 and its review, so it hears the two
rules. Codex (`gpt-5.6-terra`, medium reasoning) then does tasks 2 to 8 and every follow-up. Codex never hears
the review.

| Arm | Task 1 and its review | Tasks 2 to 8 |
|---|---|---|
| **codex** | Claude Code, plain | Codex, plain |
| **codex-celis** | Claude Code with celis | Codex with the celis plugin, on the same `CELIS_HOME` |

Every Codex session is a new process:

```
codex exec "<prompt>" --json --ephemeral --model gpt-5.6-terra \
  --dangerously-bypass-approvals-and-sandbox --dangerously-bypass-hook-trust
```

Each run has its own `CODEX_HOME`, with only the login, the model and, for codex-celis, the plugin installed
from the local repo with `codex plugin add`. So the user's Codex config, memories and `AGENTS.md` stay out.
Codex ran on a ChatGPT plan, so its sessions have no dollar cost; the table gives minutes and tokens.

#### Result (1 run per arm)

| Measure, tasks 2 to 8 | Codex | Codex + Celis |
|---|---|---|
| Rule faults, single shot | **19** | **2** |
| png and jpg files added | 15 | 0 |
| Descriptions over 160 characters | 4 | 2 |
| Follow-up rounds | 4 | 2 |
| Desktop likeness, single shot | 0.77 | 0.79 |
| Minutes, with follow-ups | 54.9 | 67.8 (+23%) |
| Input tokens, cached included | 10.9M | 17.9M (+63%) |
| Output tokens | 89k | 112k (+25%) |

| Task | Codex faults | Codex + Celis faults |
|---|---|---|
| 2 `/company` | 8 webp, 1 description | 1 description |
| 3 `/customers/qa-tech` | 1 webp, 1 description | 0 |
| 4 `/customers/lingo-dev` | 3 webp, 1 description | 1 description |
| 5 `/regions` | 0 | 0 |
| 6 `/partners` | 0 | 0 |
| 7 `/compute` | 0 | 0 |
| 8 `/customers/flick-games` | 3 webp, 1 description | 0 |

#### How to read it

- The webp rule reached Codex in full: 0 png or jpg files in 7 tasks. Claude Code wrote the rule; Codex read it
  from `celis past` at the start of each task.
- The 2 faults left are descriptions that the page already had. The rule says "When writing a meta description",
  so Codex kept the long ones it did not write.
- On Codex, celis costs time and tokens, the opposite of Claude Code. Codex has no subagent, so it does the
  past step in its own context, and each task takes more turns.
- Likeness is about the same. `/partners` is the exception: 0.34 with celis, 0.54 without.

#### The fix the bench found

The first codex-celis run made 6 faults. On task 8, the prompt hook asked Codex to "run the celis:past agent".
Codex has no plugin agents, so it ran `celis:past` as a shell command, decided that celis was not installed,
and made the same 4 faults as plain Codex. Commits `3669c92` and `15c8bfb` make the prompt hook of Codex and
Gemini CLI ask to do what `past.md` says. The run above uses that fix. The first run is in
`results/iter/codex-celis-1/`.

### How celis got there

The bench found two faults in celis. We fixed them and ran the celis arm again each time. plain and static did
not change.

| Run | celis | Rule faults, tasks 2 to 8 |
|---|---|---|
| Pilot | before `06a14d9` | the review rules did not reach later tasks |
| Iteration 1 | `b8d4be2`, as released | 22, the same as plain |
| Iteration 2 | card edit only | stopped after task 1: no rule written |
| Iteration 3 | prompt hook line | 3 |
| This run | prompt hook line | 0 |

1. **Closed trails hid their rules.** A rule scoped to a trail left `celis past` when the trail closed. Commit
   `06a14d9` lists a closed trail's rules under its line in `celis past`.
2. **The review session wrote no rule.** The card said "a review … gets none", and nothing asked the agent to
   keep the rules the person gives. A card edit alone did not change what the agent did. The line that
   `celis prompt` prints at a new ask now ends with: "Write each rule this prompt states (review feedback, a
   convention, a correction) with celis decide before you change a file." After this change, the review
   session wrote both rules, and later tasks kept them.

### Limits

- 1 run per arm. The next step is 3 runs per arm.
- One repo, one model (`claude-sonnet-5-5`) and 8 tasks.
- We chose the webp and description rules so that a fault is the easy path. The real reviews of these PRs are
  bot-written, and no arm broke them.
- The prompt hook change was tuned on this bench. It is not tested on rules it was not tuned on.
- The released celis (`b8d4be2`) made 22 faults, the same as plain. The result needs the change above.
- The cross-agent result has 1 run per arm and one handoff direction: Claude Code to Codex.

### Run it

You need macOS or Linux, Node 22 or later, Python 3, `gh` signed in to GitHub, Claude Code, Codex CLI signed in for the
cross-agent arms, and `cwebp` is optional. A run of 3 arms takes about 3 hours and about $10.

```bash
git clone https://github.com/celishq/benches && cd benches/landing
(cd checks && npm install && npx playwright install chromium)
git clone --filter=blob:none --shallow-since=2026-06-01 https://github.com/supabase/supabase cache/supabase
python3 selftest.py         # checks the tasks and rules with no agent
python3 run.py gold 1 && python3 run.py base 1

export CELIS_BIN=<folder with celis> CELIS_PLUGIN=<trails repo>/plugins/celis
for arm in plain static celis; do python3 run.py $arm 1; done
python3 report.py plain static celis

export CELIS_REPO=<trails repo>    # codex-celis installs the plugin from it
for arm in codex codex-celis; do python3 run.py $arm 1; done
python3 report.py codex codex-celis
```

`data/` holds the tasks, the rules and the frames. To build them again, run `python3 tasks.py` (tasks from
the GitHub API) and `python3 frames.py` (the merged pages, rendered on the dev server).

`content:build` in `apps/www` calls the GitHub API. The harness reads a token from `gh auth token` and passes it
to the dev server and to each agent session as `AGENT_SKILLS_GITHUB_TOKEN`. Set that variable to a read-only
token first if you do not want the agent to see your own token.
