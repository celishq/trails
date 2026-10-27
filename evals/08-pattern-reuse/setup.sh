#!/bin/sh
set -e
sh "$(dirname "$0")/../../scripts/eval-celis.sh"
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
celis pattern --scope me -- "When code handles money: keep amounts as integer cents, never floats, because float rounding broke the invoice totals." >/dev/null
