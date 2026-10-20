# Raycast bench: rules that hold across projects

## Goal

Prove what a `CLAUDE.md` cannot do by itself. A reviewer gives a rule once, on one Raycast extension. The
bench counts whether later tasks keep the rule:

1. in other extensions (other projects);
2. with no person editing a file;
3. among many rules that do not apply.

Claude Code arms only, for now. The Supabase landing bench moves to `evals/landing/` as a smaller extra case.

## Source

`raycast/extensions`: about 1,900 merged PRs a month since Jul 2026, 161 authors in 2 weeks. Staff and the
Greptile review bot repeat the same convention comments on many PRs (mined in the scratchpad,
`mine-plugins/ray_all.jsonl`).

## Arms

| Arm | Memory |
|---|---|
| plain | none; auto memory off |
| auto | Claude Code auto memory on (built in, per project) |
| self | a user-level `CLAUDE.md` that says "add each review point here as a rule" |
| oracle | the bench writes each review rule into the user-level `CLAUDE.md` |
| celis | the celis plugin, built from `HEAD` |

Each run has its own Claude config folder and its own celis home, so no run sees another run.

## Phase 1: tasks and rules → verify: `selftest.py` passes

1. Pick 10 merged PRs from Jul 2026 on, each in a different extension, written by a person: about 4 new
   extensions and 6 updates. Each needs a clear description, builds, and has room to break the rules
   (an API call, a view, preferences).
2. A task is a new git repo that holds only the extension folder at the PR's base (empty for a new
   extension). The prompt is the PR's description. So each task has its own path, its own auto memory and
   its own project `CLAUDE.md`.
3. Rules (6–8), each from a mined comment that repeats on 3 or more PRs, each checked by a script:
   - CHANGELOG entry with `{PR_MERGE_DATE}`, not a date;
   - `showFailureToast` from `@raycast/utils`, not a hand-made failure toast;
   - no hand-written `Preferences` type;
   - no `navigationTitle` on the root command;
   - no `console.log` left in `src/`;
   - no test or scratch files in the extension folder.
4. Reviews: after task 1 and task 3, a review session gives the rules that the real PR broke. Tasks 4–10
   test them.
5. Score: for each rule and task, `kept`, `broken` or `no chance` (the task has nothing the rule acts on).
   The bench also records `ray build` and `ray lint` and the cost. A follow-up review (max 3) runs while a
   rule is broken, as in the landing bench.
6. `selftest.py`: each base commit checks out, and each gold PR builds. The checks score gold on its own
   rules as expected.

## Phase 2: pilot → verify: plain breaks 3 or more rule-tasks

1. Run plain and celis, 1 run each.
2. When plain breaks fewer than 3 rule-tasks, change the rules or tasks (the easy path must be the
   fault) and run again.

## Phase 3: full runs → verify: `results/<arm>.jsonl` has 50 lines for each arm

1. 5 arms × 5 runs, 2 jobs at a time, all on one celis commit.
2. Cost: the pilot gives the cost per run. I ask before this phase when the total is over $150.

## Phase 4: many rules → verify: oracle and celis both start with 100+ rules

1. Add about 100 real rules from Home Assistant, Django and Homebrew reviews that do not apply to Raycast.
   celis gets them as `me` rules, oracle and self in the user-level `CLAUDE.md`.
2. Rerun oracle and celis, 5 runs each.

## Phase 5: move the landing bench → verify: `run.py gold 1` scores 0 faults from `evals/landing/`

1. Copy the code and data to `evals/landing/`, move `cache/` with `mv`, git-ignore the cache and
   transcripts, and remove `landing/` from `celis-benches`.

## Phase 6: report → verify: `RESULTS.md` matches `report.py`

1. Per arm: share of rule-tasks broken (mean and min–max over runs), follow-up rounds, cost.

## Open questions

1. Is the auto memory arm possible in `claude -p`? A check is running.
2. The full runs cost is not known until the pilot.
