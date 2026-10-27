#!/bin/sh
set -e
sh "$(dirname "$0")/../../scripts/eval-celis.sh"
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
id() { head -1 | sed 's/.*"id":"\([0-9a-f]*\)".*/\1/'; }
printf '# shop\n' > README.md
shown=$(celis pattern --json -- "When a price is shown
Forces: the price is kept in integer cents, and the page needs a short text.
Do: divide the cents by 100 and format the result with str().
Because: it is the shortest code." | id)
step=$(celis step add --uses "$shown" --json -- "Write show_price(cents) in price.py" 2>/dev/null | id)
printf 'def show_price(cents):\n    return "$" + str(cents / 100)\n' > price.py
celis step done "$step" --as-planned --file price.py -- "show_price(1999) gives \$19.99" >/dev/null 2>&1
printf '%s\n' "$step" > step.id
