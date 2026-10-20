# CLAUDE.md size curve and self-kept CLAUDE.md: report of 20 Oct 2026

**Result:** a big CLAUDE.md did not drift. With the 5 review rules hidden among up to 400 rules (51 KB),
Haiku, Sonnet and Opus broke 0–2% of them. celis drifted: 8% (Sonnet), 14% (Opus) and 26% (Haiku) at 400
rules. The cause is a cap in `celis grep`, not the models. An agent that keeps its own CLAUDE.md broke as few
rules as celis, but it never wrote down the second review: 2 of the 5 rules were missing from its file in both
models. The other gain for celis that held is cost: a big CLAUDE.md makes every session dearer, and celis
stays flat.

All numbers are 1 run per point.

## 1. Self-kept CLAUDE.md against celis

The `self` arm tells the agent to add each review point to a user-level CLAUDE.md and to follow it in every
project. Same 10 tasks, same reviews, 43 rule checks.

| Model | self | celis (`db82b7d`) | plain |
|---|---|---|---|
| Sonnet | **0**, $1.90 | 3, $2.70 | 6, $2.60 |
| Haiku | **3**, $0.50 | 3, $0.60 | 16, $0.70 |

The counts are close, but the files are not. Both self-kept files hold only the 3 rules of the first review.
After the second review, both agents fixed the code but wrote neither data-hooks nor no-tests into the file.
Haiku then broke no-tests on 3 of 8 later tasks, the only rule it broke. Sonnet broke neither, which suggests
that useCachedPromise and no tests are its default habits for a Raycast extension, so they do not test its
memory. celis wrote all 5 rules in both models.

So "someone must write it" holds in part: told to keep the file, the agent kept the first review and dropped
the second. With 1 run, this is a sign, not a result.

## 2. The curve

`run.py <oracle|celis> 1 --rules N --model <m>`: the 5 rules from the start, among N − 5 real Raycast,
TypeScript and React rules (`data/distractors.json`). The oracle arm holds them in the user CLAUDE.md, and
the celis arm holds them as `me` rules. No reviews and no follow-ups: 10 tasks, 50 rule checks.

Rules broken, of 50:

| Rules held | 5 | 25 | 50 | 100 | 200 | 400 |
|---|---|---|---|---|---|---|
| Haiku, CLAUDE.md | 0% | 0% | 0% | 0% | 2% | 0% |
| Haiku, celis | 0% | 2% | 6% | 16% | 16% | **26%** |
| Sonnet, CLAUDE.md | 0% | 0% | 0% | 0% | 2% | 2% |
| Sonnet, celis | 0% | 0% | 2% | 2% | 4% | **8%** |
| Opus, CLAUDE.md | 0% | 0% | 0% | 0% | 0% | 2% |
| Opus, celis | 0% | 0% | 4% | 10% | 6% | **14%** |

Cost of the 10 tasks:

| Rules held | 5 | 400 | Change |
|---|---|---|---|
| Haiku, CLAUDE.md | $0.46 | $0.99 | +115% |
| Haiku, celis | $0.48 | $0.47 | flat |
| Sonnet, CLAUDE.md | $1.68 | $3.08 | +83% |
| Sonnet, celis | $2.13 | $1.93 | flat |
| Opus, CLAUDE.md | $5.95 | $9.11 | +53% |
| Opus, celis | $7.66 | $7.18 | flat |

celis costs 4–29% more than the CLAUDE.md at 5 rules, and 21–53% less at 400. The two lines cross at about
100 rules for Sonnet and 200 for Opus. `python3 curve.py` prints the table, and `results/curve.svg` shows both
curves.

## 3. Why celis drifted

At 400 rules on Haiku, the changelog rule broke on 10 of 10 tasks. In every task:

1. The `celis:past` agent ran `celis grep " live {me} "`, as `agents/past.md` says.
2. celis cut that output at about 6,500 bytes (`LINES` in `crates/celis/src/command/grep/found.rs`), which is
   about 40 of the 400 rules. The changelog rule was never in the part shown.
3. The past agent's searches use the task's words ("domains", "API"). The changelog rule's "When" is "a Raycast
   extension changes", so no search found it.
4. The rule never reached the main agent, so it wrote a date in the changelog.

The rules that broke most are the ones that apply to every task: changelog, failure-toast and no-tests. A
rule whose "When" names a topic (data-hooks: "loads data") was found by the topic search, and broke least.

## 4. Fixes for celis

1. **Never cut a live rule.** Give rules no byte cap in `celis grep`. Cut only steps, proofs and changed
   texts. 400 rules are 51 KB, which a session can take, and the curve shows models follow that much.
2. **Bring the rules without a model in the way.** Today Haiku picks what to keep from a cut list. Have
   `celis prompt` (or `step start`) print the live `me` and `project` rules whose "When" matches this
   project and the ask, chosen by celis. The past agent then looks only for past steps.
3. **Match a rule to the project, not only to the ask.** A "When a Raycast extension changes" rule holds for
   every task in a folder with `@raycast/api` in `package.json`. Keep for each rule the facts of the project
   it came from (the stack, the files), and bring every rule whose facts match, whatever words the ask uses.
4. **Merge rules.** The `tidy` skill can fold near-duplicates and old rules, so the store stays small.

Fix 1 is small, and it alone should bring celis to the CLAUDE.md line. Fixes 2 and 3 keep the cost gain.

## 5. What this means for the post

- Drop "a CLAUDE.md does not scale": at 400 rules, all three models still followed it.
- Soften "someone must write it": an agent told to keep its own CLAUDE.md missed the whole second review, but
  broke about as few rules as celis. Run `self` 5 times, with rules that are not the model's default habits,
  before you claim it.
- Keep the cost claim: a CLAUDE.md with 400 rules made every session 53–115% dearer, and celis stayed flat.
  This holds once fix 1 brings celis's breaks down to the CLAUDE.md line.
- The gains this bench does not test are still open: rules from Claude Code reaching Codex (the landing
  bench), past steps reused across projects, and trails shared with a team.

## Limits

- 1 run per point, and the Sonnet arms failed `ray build` on 1–2 of 10 tasks in both arms.
- 5 target rules, all short and mechanical. Harder or conflicting rules may drift sooner in a big
  CLAUDE.md.
- The filler rules are real conventions but do not conflict with each other or with the targets.
- celis is the merged `main` (`ff793c6`), not the version of the earlier pilots.

## Cost

About $120 in all: Haiku $6, Sonnet $25, Opus $88, and the self arms $2.40.
