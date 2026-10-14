# A clearer celis tree

Trail `891b126e`. The work starts after `feat-add-stats` (trail `f1943ab0`) is merged to main. That branch
rewrites `tree.rs`, moves the tree's output rules into `spec/trails/02-cli.md`, removes steers and adds the
states reached and revised. Each phase changes the spec first, then the tests, then the code.

## Goal

The tree shows two things: the progress of the agent's work and its lessons. The form follows that
function: the work is a log of steps in time order, with the step now at the front; the lessons sit where
they came from, with a count at the root.

## Phase 1: the layout, for the pipe and the terminal

- Two groups. `ACTIVE`: each active trail with every step under it. `DONE`: each trail that ended today,
  one line. Earlier days are only in `--all`, which keeps its day lines and prints its trails as `DONE` lines.
- `tree <id>` opens one trail, of any state, as an active trail prints. There is no `--open` flag.
- An active trail has a head of two lines: its mark, title (wraps, never cut) and project; then its progress
  (`6 of 8`), its worked time, `§ <n> rules` and its id. A blank line comes before its steps.
- Steps keep their depth. A level is two spaces, with a dim `│` guide in place of `├─` and `└─`.
- A done line: mark, title, project, then `<n> steps` or `<done> of <all>`, the rule count, its worked
  time and its id. A pitfall is a decision that names no step, so it counts as a rule.
- Worked time is the time steps were working. It is the only time; start times go to `--full`.
- Rules stay at the root: a count in the short tree, the list under the head with `--full`.
- `~ <changed>` prints under its step, cut to the width in the short tree.
- Marks: `○` open, `◐` working, `✓` done, `↻` revised, `✗` dropped, `!` expired. A dropped trail's reason
  prints after its mark, cut to the width.
- Times and ids end at the right edge, in one column.
- Brightness: the working step bright, open steps normal, done work and details dim. One accent for the
  work now, red only for `✗` and `!`. No green. Without colour the lines are the same.

Verify: `trail_tree.rs`, `step_under.rs`, `step_drop.rs`, `closing.rs` and `tree/tests.rs` change first and
fail; `make verify` passes; `celis tree --all | cat` reads as the mocks of the thread.

## Phase 2: the live view

- A working step's mark is a spinner on a terminal; the pipe keeps `◐`.
- The head of an active trail shows `active <time> ago`, from its newest record.
- A step or trail that changed state prints bold for 2 seconds; a trail that ends moves to `DONE`.
- The view draws on the alternate screen, each frame over the last with no clear, so lines do not jump.
  A frame taller than the screen ends with `… <n> more lines`. On an interrupt or `q`, the last frame
  prints on the main screen and stays in the scrollback.

Verify: `trail_watch.rs` drives frames with a fixed clock; `make verify`.

## Phase 3: small widths

- Below 80 columns the view leaves out, in order: ids, then project names, then times.

Verify: frame tests at 60, 80 and 160 columns.

## Phase 4: keys

- `↑ ↓` select a trail, `enter` opens or closes it, `f` turns `--full` on and off, `a` turns `--all` on
  and off, `q` quits. A dim last line names the keys. Off a terminal there are no keys.
- Key input reads stdin with its echo and line mode off, through `libc`, as the watch already reads the
  terminal's size and signals. No new dependency.

Verify: tests feed keys to the view and check the frames; `make verify`.
