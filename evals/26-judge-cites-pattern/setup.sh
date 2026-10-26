#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
printf 'def total(cents):\n    return sum(cents)\n' > cart.py
celis pattern -- "When a refund is computed: round it down to the cent, because the shop never pays back more than it took." >/dev/null
