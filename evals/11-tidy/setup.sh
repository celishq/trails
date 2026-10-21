#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
celis decide --scope at:old/prices.py -- "When you change old/prices.py: round each price down, because the old shop rounded down." >/dev/null
celis decide -- "When code handles money: keep amounts as integer cents, because floats lose a cent." >/dev/null
celis decide -- "When you store a price: store it in whole cents, never as a float, because float sums drift." >/dev/null
celis decide -- "When you indent Python: use tabs, because the first editor used tabs." >/dev/null
celis decide -- "When you indent Python: use four spaces, because PEP 8 asks for spaces." >/dev/null
for n in 1 2 3 4; do
  trail=$(celis step add --new -- "Add a changelog entry for release 1.$n" 2>/dev/null)
  step=$(celis step add --under "$trail" -- "Write the entry under a new version heading in CHANGELOG.md" 2>/dev/null)
  celis step done "$step" --as-planned -- "CHANGELOG.md has a 1.$n heading with the entry." >/dev/null 2>&1
done
mkdir -p tests
printf 'from cart import total\n\n\ndef test_total():\n    assert total([150, 250]) == 400\n' > tests/test_cart.py
printf '[project]\nname = "shop"\n\n[tool.ruff]\ntarget-version = "py311"\n' > pyproject.toml
printf '# shop\n\nAgents read this file before they work here.\n' > AGENTS.md
long=$(celis decide -- "When you write Python: keep each line at most 100 characters, because the team reviews code side by side.")
celis decide -- "When you finish a change: tell the person which files you changed and why, because they review each change by hand." >/dev/null
rule=$(celis decide --path "test/*.py" -- "When you write a test: give each test one assert, because a failure must name one cause.")
trail=$(celis step add --new -- "Add tests for the cart" 2>/dev/null)
case "$(uname)" in Darwin) home="$HOME/Library/Application Support/Celis" ;; *) home="$HOME/.local/share/celis" ;; esac
home="${CELIS_HOME:-$home}"
mkdir -p "$home"
for n in 1 2 3; do
  step=$(celis step add --under "$trail" --at tests/test_cart.py -- "Add test $n to tests/test_cart.py" 2>/dev/null)
  celis step done "$step" --as-planned -- "tests/test_cart.py has test $n, and pytest passes." >/dev/null 2>&1
  printf '{"at":%s,"session":"s%s","rule":"%s","step":"%s"}\n' "176000000$n" "$n" "$long" "$step" >> "$home/given"
  printf '{"at":%s,"session":"s%s","rule":"%s","miss":true}\n' "176000010$n" "$n" "$rule" >> "$home/given"
done
celis step done "$trail" --as-planned -- "tests/test_cart.py has 3 tests, and pytest passes." >/dev/null 2>&1
