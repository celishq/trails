#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
celis pattern --path old/prices.py -- "When you change old/prices.py: round each price down, because the old shop rounded down." >/dev/null
celis pattern -- "When code handles money: keep amounts as integer cents, because floats lose a cent." >/dev/null
celis pattern -- "When you store a price: store it in whole cents, never as a float, because float sums drift." >/dev/null
celis pattern -- "When you indent Python: use tabs, because the first editor used tabs." >/dev/null
celis pattern -- "When you indent Python: use four spaces, because PEP 8 asks for spaces." >/dev/null
printf 'Fresh beans. Roasted Monday. Gone by Friday.\n' > coffee.md
printf 'Our carefully curated selection of exquisite teas offers a truly unforgettable experience.\n' > tea.md
printf 'Warm bread. Baked at six. Sold by noon.\n' > bread.md
celis pattern --path "*.md" --good coffee.md --bad tea.md -- "When a shop blurb is written: keep it to three short plain sentences, because the person liked the coffee blurb." >/dev/null
celis pattern --path "*.md" --good bread.md -- "When page copy is written: use short, plain sentences with no praise words, because the person picked the bread blurb." >/dev/null
mkdir -p tests
printf 'from cart import total\n\n\ndef test_total():\n    assert total([150, 250]) == 400\n' > tests/test_cart.py
printf '[project]\nname = "shop"\n\n[tool.ruff]\ntarget-version = "py311"\n' > pyproject.toml
printf '# shop\n\nAgents read this file before they work here.\n' > AGENTS.md
lines=$(celis pattern -- "When you write Python: keep each line at most 100 characters, because the team reviews code side by side." | head -1)
told=$(celis pattern -- "When you finish a change: tell the person which files you changed and why, because they review each change by hand." | head -1)
for n in 1 2 3; do
  trail=$(celis step add --new -- "Change the cart, part $n" 2>/dev/null)
  ask=$(celis step add --under "$trail" --recalled "$lines" --recalled "$told" --uses "$lines" --uses "$told" -- "Change cart.py, part $n" 2>/dev/null)
  celis step done "$ask" --as-planned -- "cart.py has part $n, and pytest passes." >/dev/null 2>&1
  celis step done "$trail" --as-planned -- "cart.py has part $n, and pytest passes." >/dev/null 2>&1
done
celis pattern --path "test/*.py" -- "When you write a test: give each test one assert, because a failure must name one cause." >/dev/null
trail=$(celis step add --new -- "Add tests for the cart" 2>/dev/null)
for n in 1 2 3; do
  step=$(celis step add --under "$trail" --at tests/test_cart.py -- "Add test $n to tests/test_cart.py" 2>/dev/null)
  celis step done "$step" --as-planned -- "tests/test_cart.py has test $n, and pytest passes." >/dev/null 2>&1
done
celis step done "$trail" --as-planned -- "tests/test_cart.py has 3 tests, and pytest passes." >/dev/null 2>&1
