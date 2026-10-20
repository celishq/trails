#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
celis decide -- "When code handles money: keep amounts as integer cents, never floats, because float rounding broke the invoice totals." >/dev/null
printf "# Conventions\n\n- When code handles money: keep amounts as integer cents, never floats, because float rounding broke the invoice totals.\n" > CLAUDE.md
