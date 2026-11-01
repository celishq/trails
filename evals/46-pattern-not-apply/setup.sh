#!/bin/sh
set -e
sh "$(dirname "$0")/../../scripts/eval-celis.sh"
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
printf '# fx\n' > README.md
celis pattern --scope me -- "When code handles money
Forces: a sum of float amounts drifts by a cent over many line items.
Do: keep amounts as integer cents, never floats.
Because: float sums broke the invoice totals." >/dev/null
