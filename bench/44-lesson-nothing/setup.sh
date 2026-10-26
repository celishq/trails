#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
export CELIS_SESSION_ID=bench
printf '# shop\n' > README.md
id() { head -1 | sed 's/.*"id":"\([0-9a-f]*\)".*/\1/'; }
say() { printf '{"session_id":"bench","prompt":"%s"}' "$1" | celis hook ask claude >/dev/null 2>&1; }
say "Rename calc to compute_total in cart.py."
ask=$(celis step add --json -- "Rename calc to compute_total in cart.py" 2>/dev/null | id)
printf 'def compute_total(cents):\n    return sum(cents)\n' > cart.py
celis step done "$ask" --as-planned -- "grep finds no calc in cart.py; compute_total([1, 2]) gives 3" >/dev/null 2>&1
printf '%s\n' "$ask" > ask.id
