#!/bin/sh
set -e
sh "$(dirname "$0")/../../scripts/eval-celis.sh"
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
printf '# shop\n' > README.md
printf 'def total(prices):\n    return sum(prices)\n' > cart.py
git init -q
git add README.md cart.py
git -c user.name=eval -c user.email=eval@example.com commit -qm "first"
