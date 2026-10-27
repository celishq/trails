#!/bin/sh
set -e
sh "$(dirname "$0")/../../scripts/eval-celis.sh"
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
printf 'def parse_age(text):\n    return int(text)\n' > age.py
printf 'the name is empty: give a name of 1 to 40 letters\n' > good-error.txt
printf 'invalid input\n' > bad-error.txt
celis pattern --path "*.py" --good good-error.txt --bad bad-error.txt -- "When an error message is written: say what is wrong and what to give instead, because the person liked a message that names the fix." >/dev/null
