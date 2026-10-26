#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
printf '# shop\n' > README.md
id() { head -1 | sed 's/.*"id":"\([0-9a-f]*\)".*/\1/'; }
trail=$(celis step add --new --json -- "Build the cart" | id)
ask=$(celis step add --under "$trail" --json -- "Write total(prices) in cart.py" | id)
printf 'def total(prices):\n    return round(sum(prices) * 1.08, 2)\n' > cart.py
celis step done "$ask" --as-planned -- "total([100]) gives 108.0" >/dev/null 2>&1
fix=$(celis step add --replaces "$ask" --json -- "Keep the cart total in integer cents" | id)
printf 'def total(cents):\n    return sum(cents) * 108 // 100\n' > cart.py
celis step done "$fix" --as-planned --by person -- "was: return round(sum(prices) * 1.08, 2) / now: return sum(cents) * 108 // 100" >/dev/null 2>&1
celis pattern --good "$fix" --scope me --path "**/*.py" -- "When code handles money: keep amounts as integer cents with integer math, because the person rewrote a float total." >/dev/null
