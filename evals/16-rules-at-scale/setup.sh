#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
n=0
while IFS= read -r rule; do
  n=$((n + 1))
  [ "$n" = 150 ] && celis decide --scope me -- "When code handles money: keep amounts as integer cents, never floats, because float rounding broke the invoice totals." >/dev/null
  celis decide --scope me -- "$rule" >/dev/null
done <<'RULES'
Never address record fields by position when they have names.
Extract shared code only after the third copy, not the second.
Report security problems through the private advisory form in `SECURITY.md`, never in a public issue.
Use realistic SKUs such as `BOLT-M6-30` in examples, not `foo` and `bar`.
Credit outside contributors by handle in the release notes.
Group related edits across files, then run the suite once, not after each small edit.
Run local benchmarks with `make bench`; it pins the fixture and warms the cache before measuring.
Run table cases as subtests so one bad row does not hide the others.
Run the lint job first and make the test matrix depend on it, so style breaks fail fast and cheap.
Keep per-file ignores in `pyproject.toml`, sorted by path.
Allow a new TODO only when it links an open issue.
For a bug fix, write a test that fails on the current code, then make it pass.
Always give a yank reason on PyPI that names the version to use instead.
Say "below reorder point", not "low stock", when the reorder rule triggered.
Reject output names that contain `..` segments or a drive letter.
When you paste command output into an issue or PR, redact anything that looks like a token or a password.
Ask reporters to share redacted inventory samples only; never ask for their real supplier data.
Let callers pass an idempotency key with a movement so a retried receipt is not counted twice.
Stock builds one pure-Python wheel tagged `py3-none-any`; never add compiled code.
All package metadata lives in `pyproject.toml`; do not add `setup.py` or `setup.cfg`.
After review starts, push follow-up commits instead of rewriting history, so reviewers can see what changed.
Do not use sleeps to wait in tests; drive the code to the state you need directly.
Take locks in a fixed order: data directory first, then item files. Never take them in reverse.
Say in the body when a commit changes the on-disk inventory layout, so reviewers know to check upgrades.
Close the issue from the PR with `Fixes #123`, not by hand.
Each example must run on its own with only `stock` installed; no shared modules between examples.
Keep each user-facing sentence to one idea and under 20 words.
Use rebase-merge for a PR whose commits were curated to stand alone.
Pin runner images to a version, such as `ubuntu-24.04`, not `ubuntu-latest`.
Do not publish a release on a Friday or before a holiday, so a fix can follow fast.
Point out copied blocks of code and ask whether one shared function would be clearer.
After each step in a guide, state what the reader should see next.
Do not use `shelve` for persistence; it is pickle underneath and has the same risk.
Do not mix `item`, `product`, and `article` in docs; `item` is the term.
The release job pins archive member times to the tag's commit time so builds stay stable.
Every ADR has four sections: context, decision, consequences, status.
Expired reservations are released by `stock sweep`, never as a side effect of an unrelated read.
Before an irreversible action, say plainly that it cannot be reversed.
The `stock` command comes from `[project.scripts]`; do not add a second entry point for the same program.
Do not add values of different currencies; reject the operation instead of guessing a rate.
Add one case with ten thousand items to code that walks the whole inventory.
Backups get the same `0o600` mode as the database they copy.
Keep the item record small: SKU, name, base unit, status, and tags. Everything else is linked by SKU.
When a reporter says a fix did not work, reopen the issue rather than filing a new one.
Validate input before you take the lock, so invalid input never holds up other writers.
Never answer an empty result with no text at all in interactive use.
Do not pin a pre-release of a dev tool on `main`.
Document the durability promise in `docs/storage.md`: a completed command survives a crash; an interrupted one leaves no trace.
Seed the fixture generator so every run produces the same rows and benchmarks stay comparable.
Do not use sleeps to wait for a lock in tests; use a barrier or an event so the test is fast and stable.
Keep review findings to what the PR introduces or worsens; file older problems as issues.
A reservation has its own lifecycle: `held`, `released`, `consumed`. Do not infer state from other fields.
Use domain words in code: `reorder_point`, `on_hand`, `reserved`, matching the glossary.
Put breaking changes at the top of the release notes with the steps a user must take.
Use fenced code blocks with a language tag such as `console` or `python`.
Test every migration against the 100k-item fixture and note how long it takes in the PR.
Agents should show the proposed commit message and wait for approval before they commit.
A backend may add an index for speed, but the result of every interface call must not change because of it.
Label a PR `breaking` when it changes a public signature, return shape or persisted field.
Show the filters that shaped a report under its title so a pasted report explains itself.
An SKU is an opaque identifier. Do not parse meaning out of its parts in domain code.
Keep the storage interface small: get, list, put, append movement, and transaction. Resist adding query methods.
Compare SKUs without regard to case, but keep the case the user first entered for display.
Hide `+1` comments as off-topic and point people to the reaction button.
Do not reach for regex where `str.startswith` or `str.split` will do.
Merge the fix on `main` first; backport only after it has passed CI there.
Group related writes in one storage transaction. A partial write must leave no trace.
Sort report rows by SKU in tests so differences point at real changes.
Tests in CI must not reach the network; anything that needs it lives in the nightly job.
Update or delete a comment in the same commit that changes the code it describes.
Prefer `sum`, `min`, `max`, `any`, and `all` over hand-written loops; they run in C.
Tag releases as `vX.Y.Z` on the merge commit in `main`; never tag a commit that has not passed CI.
Move code and change code in separate commits so the diff of the change stays readable.
If a fix grows past twice the size you expected, step back and look for a smaller one.
One topic per page under `docs/guide/`; split a page when it covers two unrelated tasks.
Ship SQL migration files as package data and read them through the `resources` API, not through `__file__`.
Do not leave a comment that is only an issue number; add a short reason next to it.
Link every PR to an issue; only small typo fixes may skip this.
Read the release notes of a tool before you merge its Dependabot bump; do not merge on green alone.
Apply locale-aware number formatting only in human-readable reports.
A location is identified by warehouse and bin, e.g. `MAIN/A-03-2`. Treat the pair as one value.
Stock uses semantic versions, not calendar versions; do not change the scheme.
The CSV backend has no WAL, so readers take a shared lock too.
A lookup that answers a question about inventory must not change anything.
Write the owner PID into the lock file so a stale lock can be detected and reported.
Use numbered lists for steps that must happen in order, bullets for everything else.
Do not add a rule to the global ignore list to silence one file; use a per-file ignore.
Schedule cron jobs at an odd minute, e.g. `17 3 * * *`, to avoid the top-of-hour queue.
Add an `os/windows` or `os/macos` label when a bug only shows on one platform.
Do not mock the item model or the quantity math; build real objects with small fixtures.
Run the secret scanner from the pre-commit hooks before you push a branch that touches sample data.
Cycle counts produce adjustment movements; they never overwrite the on-hand figure directly.
After a fix ships, comment on the issue with the version that contains it.
Never hand-edit generated files; change the source and run `make gen`.
Publish to PyPI only through the trusted-publishing release job; never upload from a laptop.
If you change the sample inventory under `examples/`, explain in the description why the old sample no longer fits.
Confirm the wheel file name before upload; a wrong tag means pip on some platforms will skip it.
Never land commits titled `wip`, `fixup`, or `tmp` on `main`; squash or reword them first.
A PR that slows a tracked benchmark by more than 10% needs a note in the description that says why.
After the rename, fsync the parent folder so the new directory entry survives a power loss.
Confirm a finished action in past tense: `Received 40 EA of BOLT-M6 at MAIN/A03/B12`.
Never call `list.pop(0)` in a loop; use `collections.deque`.
Check that a write path leaves the old file intact when the disk fills mid-write.
Never expose repository secrets to jobs triggered by forks.
A command on the 100k-item fixture must stay under 200 MB of resident memory.
Prefix module-private names with one underscore; do not use two.
Remove branches that no caller can reach once you have proved it with grep.
Ask a maintainer to review any change that adds, removes or alters a public name.
Do not re-run a red job until it passes and then merge. Find out why it failed first.
Avoid merging risky storage changes late on a Friday when nobody can watch `main`.
Stream rows from the backend one at a time; do not load the whole inventory into memory to answer a question about one SKU.
If a perf change makes the code hard to follow and the win is under 10%, revert it.
Keep the dependency graph between packages pointing one way: commands -> reports -> models <- storage.
The CLI may use only the public library surface, so every CLI feature is also usable from Python.
Never delete a lock held by a live process, even if the wait is long.
Let the reviewer resolve the threads they opened; resolve a thread yourself only when it was a pure nit you applied.
Each release checks that a database made by the previous release opens and migrates cleanly.
Delete unused functions and branches instead of keeping them for later.
Fixture data uses obviously fake credentials such as `hunter2-not-real`; keep it that way.
Do not bump the package version in a feature PR; the release PR does that.
A model change that alters what is stored needs a matching update to the sample data in `examples/`.
SKU strings repeat across millions of movements; intern them when loading large ledgers.
Look for loops that read the whole inventory per item; ask for one pass instead.
Do not add a new abstraction for a single caller. Wait for the second real use before you extract it.
Mark every experimental page in the docs with a visible "Experimental" banner.
Cap item names at 120 code points and say so in the reference docs.
A rollback must leave the data directory byte-for-byte as it was before the transaction.
When closing a duplicate, add the reporter's words to the original issue title or body so search finds it.
Separate the steps of a function with single blank lines; never use two inside a function.
Upload the coverage report as a job artifact; do not send it to an outside service.
Add every new docs page to the site navigation in the same PR that creates it.
When you hand work to another agent, list the files changed, checks run, and open questions.
Keep the per-location stock view and the per-item stock view backed by the same function.
Bots must not push to PR branches. Suggest changes in a review comment instead.
Memoise lookups for the duration of one command run through a plain dict passed down, not a hidden global.
Side effects live at the edges: storage below, the terminal above. The middle is plain functions over models.
Never commit `__pycache__` folders or `.pyc` files.
Label an issue `needs-repro` when you cannot reproduce it, and say what you tried.
Keep examples covered by `make examples` so CI catches a broken one.
Do not add an abstraction for a second use case that does not exist yet.
Stock is a regular package; do not turn it into a namespace package.
Add the commit SHA of a mass reformat to `.git-blame-ignore-revs`.
Accept a follow-up issue for non-blocking improvements instead of growing the PR.
Keep caches small. A cache that takes longer to fetch than to rebuild is worse than no cache.
Run `make mdlint` before you push; fix findings rather than disabling the rule.
Name the exact target in a confirmation, for example `Delete location MAIN/A03/B12 and its 14 items?`.
Changes under `.github/` need review from a member of the `maintainers` team.
Close backend connections at the end of each operation scope; use a context manager.
Support every CPython release that upstream still maintains, and nothing older.
If you find an unrelated bug while you work, open an issue or a separate PR for it.
If a caller needs to walk rows twice, materialise them once with `list()` at the call site, not inside the backend.
Supplier records hold a company name and a reference code only, never a person's phone number or email.
Do not merge a PR until every required check is green. Never ask an admin to bypass branch protection.
The version lives in one place, `src/stock/__init__.py`; the backend reads it from there.
Return an iterator from queries that can yield more than a few thousand rows; offer a list form only if a caller needs it.
Sign release and tag commits with a verified GPG or SSH key.
Do not mix mass reformatting with behaviour changes in one PR.
Attach a `SHA256SUMS` file for the sdist and wheel to each GitHub release.
Give each reversing movement a link to the movement it cancels.
Write an upgrade guide page for every release that changes the on-disk format.
Every bug fix ships with a regression test that would have caught the bug.
Thank the reporter, stay polite, and do not argue in issues; move long design debates to a call.
Do not add backend features that only the in-memory backend can do. Every backend is a first-class citizen.
The design assumes one writer at a time per data file. Do not add code paths that write without the lock.
Do not connect to the backend until a command actually reads or writes inventory.
Link a domain word such as `reservation` or `bin` to its glossary entry the first time a page uses it.
In the PR, explain why the bug happened in one or two sentences, not only what changed.
Do not use a real company's SKU list as sample data, even if it is public.
Use annotated tags, not lightweight ones, so the tagger and the message stay in the history.
Keep fixture files small; five items and a few movements show most behaviours.
Let the labeler bot add area labels from paths; correct them by hand if they are wrong.
Do not take the `hacktoberfest` label; low-effort PRs get closed without review.
CI must run the same commands as `make check`; do not put steps in the workflow that developers cannot run.
Spell names out: `quantity`, not `qty`; `location`, not `loc`.
Prefer a plain loop over a clever one-liner when the one-liner needs a second read.
Never rebase `main` or any release branch; those histories are fixed.
Do not ping reviewers more than once a week on the same PR.
Mention unrelated bugs you notice in the report, but do not fix them in the same diff.
Do not leave tests commented out; skip them with a reason or delete them.
Never rewrite published history to remove a file; ask a maintainer to rotate the secret and follow the incident steps.
Refactor in small steps that each leave the code working, not in one large rewrite.
For any feature that reads user files, add a short threat note to the PR: what input, what trust, what limit.
Every Makefile target has a one-line `##` note so `make` with no target lists them.
Keep the roadmap in a GitHub project board, not in the issue tracker labels.
In the report, list the exact commands you ran and whether each one passed.
Cap the number of rows a single batch can add so a malformed file cannot fill the disk.
A reader must never see a half-written data file. If you cannot make a write atomic, do not ship it.
Use `functools.lru_cache` only on pure functions of immutable inputs.
Check that copied third-party code keeps its license notice and is listed in `NOTICE`.
State in the reference docs when a returned iterable is single-pass.
Keep docs-build tools in their own `docs` group so the test job stays lean.
Do not commit editor folders such as `.vscode/` or `.idea/`.
Record a transfer as two movements, an issue at the source and a receipt at the target, sharing one transfer id.
When you move a public model to a new module, keep the old path working for one minor release.
Stop scanning as soon as the answer is known; `any()` over a generator beats building a list first.
Use `uv lock` to regenerate the lock file; other resolvers write a different format.
Run `codespell` on docs and source; add real words to `.codespell-ignore`, not to each file.
Bound every lock wait; a command must not hang forever behind a stuck writer.
Open a draft PR early to share direction; mark it ready only when checks pass and the description is complete.
Do not let one class own items, locations, and movements at once; split it by concept.
Name each new transitive package in the PR description when a lock change adds one.
Do not denormalise supplier names into movements. Keep the supplier code and look the name up.
Treat the SQLite `-wal` and `-shm` files as part of the database for permissions and backups.
Do not let callers register backends by mutating a module-level dict; expose a registration call.
Create release tags as annotated tags: `git tag -a v1.5.0 -m "stock 1.5.0"`.
Put scratch scripts and sample data in the session scratch folder, never in the repo tree.
Assign yourself before you start work so no one else starts the same fix.
Try an unclear behaviour in a short scratch script before you build a change on top of it.
Minimum order quantity and pack multiple live on the item-supplier link, not on the item.
Put an upper bound on the size of a file that stock reads into memory, and document the bound.
The hot paths are quantity lookup, movement append, and the on-hand report. Treat changes there with extra care.
Update the Python version classifiers in the same PR that changes `requires-python`.
Write headings in sentence case: `Moving items`, not `Moving Items`.
Run `make lint` before review; a reviewer should never point out what a linter can find.
Treat a stale lock as a real lock until its owner process is confirmed gone.
Request a CVE through the GitHub advisory for any fix that changes what an attacker can do.
Prefer explicit fields over positional tuples in stored records, so a reader can tell what each value is.
Release every lock on every path out of a write, including early returns.
Generate the large fixture with `make fixtures`; do not commit files over 5 MB to the repo.
Do expensive pure work, such as parsing a large input file, before taking the write lock.
Do not paste a generated wall of text into the PR description; keep it to what a reviewer needs.
Never enable SQLite extension loading; stock does not need it and it widens the attack surface.
File an issue for each follow-up you defer, and link it in the PR.
Allow negative on-hand counts only when the location opts in; otherwise reject the movement.
Keep one workflow per purpose: `ci.yml`, `release.yml`, `docs.yml`, `nightly.yml`.
Remove a function that only forwards to another function with the same arguments.
Every `# noqa` suppression carries a short reason after it.
Use `fetchmany(1000)` in loops over large result sets rather than `fetchall()`.
Keep the public surface of `stock` small. Anything not documented in `docs/api.md` is internal and may change.
For a regression, run `git bisect` between the last good tag and `main` and post the bad commit.
Remember that the last matching CODEOWNERS pattern wins; put specific paths below general ones.
Commit a lockfile refresh by itself with the subject `chore(deps): refresh lockfile`.
Each report lives in `stock/reports/` and takes models in, returns rows out. Rendering is done by the caller.
Avoid short abbreviations in names; write `quantity`, not `qty`. SKU is the one accepted short form.
Do not follow symlinks when you write exports into a shared folder; refuse the write instead.
Keep the FAQ to questions users actually asked; cite the issue number next to each entry.
Name domain concepts after the words warehouse staff use: on hand, reserved, available, receipt, shipment.
Question every new runtime dependency; `stock` uses the standard library only.
Model negative stock as a visible state with its own report, not as a hidden clamp to zero.
Ask reporters for the item and movement counts of their data set; most slowness reports depend on size.
Reports must not call each other. Share the calculation through a function in `stock/models/` instead.
Keep a backport as small as the fix; do not pull in refactors that the fix happened to sit on.
Use comprehensions for row transforms; they are as fast as `map` and easier to read.
After merge, run the reproduction once on `main` to confirm the fix landed.
Every breaking change ships with a migration note in `docs/upgrading.md`.
Put an issue in a milestone only when someone has committed to ship it in that version.
Use the merge queue for `main`; the queue re-runs required checks on the merged result.
Put domain rules on the model that owns the data, not in the command that happens to need them.
Keep piped results free of banners and decoration so they can feed another tool directly.
Reserved quantity can never exceed on-hand quantity at the same location.
Do not shadow built-in names such as `id`, `list`, `type`, or `input`.
Name uploaded artifacts with the job and matrix cell, e.g. `coverage-py312-ubuntu`.
Use `collections.Counter` or a dict to sum quantities per key in one pass.
A serial number belongs to exactly one item and is unique across the whole data directory.
Write comments in English, even in files that handle localised item names.
Destructive commands such as `purge` must name what they will remove and ask for confirmation first.
The wheel carries no tests, docs or examples; only the sdist does.
Run the test job on every supported Python version, from the oldest in `requires-python` to the newest.
Do not cache derived quantities in the model. Recompute from the source data or invalidate on every write.
Make sure `.claude/worktrees/` is in `.gitignore` before you add a worktree.
Run SQLite in WAL mode so a long report does not block a concurrent `stock add`.
Ask for the `stock` version, Python version, OS, and the exact command before you label a bug confirmed.
Delete the remote branch after the PR merges; GitHub does this when you enable auto-delete.
Separate the subject from the body with one blank line.
Jobs that run on fork PRs must not have access to repository secrets.
Label an issue `good first issue` only if it has a clear scope, the files to touch, and a test to write.
Open any sample inventory file the PR changes and confirm the numbers add up.
Every PR description ends with a short test plan listing the commands you ran.
Do not mix regenerated files with hand edits in one commit unless the edit caused the regeneration.
Triage open security alerts every release; an unread alert is not evidence of zero findings.
Ask for the source of any binary file a PR adds.
Break a chain of more than three method calls into named intermediate values.
Outside contributors must accept the contributor agreement before a maintainer reviews the PR.
Keep each test short and about one behaviour; split a test that checks three things.
`stock` is I/O bound on small data and CPU bound on large ledgers; profile before you assume either.
Mark a breaking public change with a `BREAKING CHANGE:` footer that names the old and the new behaviour.
Ask why if a PR skips a CI job that normally runs for the changed paths.
Maintainers may close a PR with no activity for 60 days; reopen it when you resume work.
A CVE in a dev-only tool is still a CI risk; bump it within the normal weekly cycle.
Avoid mixins in the domain layer. Shared behaviour goes into a plain function both models call.
Package data lives inside `src/stock/`; nothing outside the package directory reaches the wheel.
Use a fixed set of reason codes for movements, e.g. `receive`, `ship`, `transfer`, `count`, `scrap`. Add new codes in one place.
Reject Windows reserved names such as `CON` and `NUL` when you build export file names.
Check that values read from inventory files are validated before use.
A change to stored data layout needs a design note in `docs/decisions/` before code review.
Write comments as full sentences with a capital letter and a period.
Add the issue number to the branch name when one exists: `fix/412-duplicate-sku`.
CI tests the built wheel, not an editable install, so packaging gaps fail early.
Put a comment on its own line above the code it describes, not at the end of the line.
Keep reorder and valuation calculations pure: inputs in, result out, no backend access.
Do not reformat code you did not otherwise change in the same PR.
Keep notes of each hypothesis you tested and its result so the next person does not repeat them.
Write a format version into every export file so later releases can read older exports.
Do not remember answers to prompts between runs; every run asks again.
Add a case for negative quantities wherever stock can be adjusted down.
A transfer between two locations is one movement with two legs; write both legs together or neither.
Commit the lock file in the same PR as the change that needs it.
Each data file carries a schema version. Readers refuse to touch a file with a newer version than they know.
Keep job output short; send long reports to artifacts rather than the job page.
RULES
