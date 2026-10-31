#!/bin/sh
set -e
sh "$(dirname "$0")/../../scripts/eval-celis.sh"
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
printf 'def parse_age(text):\n    return int(text)\n' > age.py
