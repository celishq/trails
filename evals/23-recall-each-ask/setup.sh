#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
printf '# shop\n' > README.md
printf 'def total(cents):\n    return sum(cents)\n' > cart.py
celis decide --situation -- "When a refund is computed: round it down to the cent, because the shop never pays back more than it took." >/dev/null
celis decide --path "*.py" -- "When Python code changes: give each function type hints, because the team reads them." >/dev/null
