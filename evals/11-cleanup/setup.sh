#!/bin/sh
set -e
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
