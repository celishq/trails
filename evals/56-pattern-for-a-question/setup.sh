#!/bin/sh
set -e
sh "$(dirname "$0")/../../scripts/eval-celis.sh"
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
printf 'def parse_quantity(text):\n    return int(text)\n' > order.py
while IFS='|' read -r name when do; do
  celis pattern -- "Name: $name
$when
Forces: a quick fix is cheap, but the next reader pays for each surprise
Do: $do
Because: the person corrected this before, and the fix held" >/dev/null 2>&1
done < "$(dirname "$0")/../55-pattern-in-a-large-store/patterns.txt"
