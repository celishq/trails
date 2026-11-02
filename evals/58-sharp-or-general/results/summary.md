# Eval 58: a concrete lesson stays sharp, a general one stays open

The case gives two lessons in one prompt: never print in this project's .py files (concrete), and an
error message says what went wrong and what to do next, in any work (general). It compares the card,
brief and extraction agent at `e70b0a8` (old) with the ones at `c3a005a` (new). Each arm ran 3 times on
claude-opus-5-5, with the celis built from its own commit, on 02-11-2026, with
`--allow-tools Bash Write Edit`.

| Grader               | Old card | New card |
| -------------------- | -------- | -------- |
| concrete-stays-sharp | 3/3      | 3/3      |
| general-stays-open   | 2/3      | 2/3      |
| fixed                | 3/3      | 3/3      |
| Cost                 | $1.03    | $1.05    |

Both arms wrote the print lesson with `--path "**/*.py"` and a Do that names `logging.getLogger`, and
the error lesson with `--scope me`. On Opus, the old card already keeps a concrete lesson sharp, so this
case does not show that the new wording changes what an agent does. Each general-stays-open miss came
from a `--good` example that quotes `greet()`, not from the pattern's text: the grader counts the
examples too.
