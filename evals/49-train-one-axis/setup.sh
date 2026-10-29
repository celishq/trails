#!/bin/sh
# Gives the store one earlier taste round, on the ground colour, with its files and its pattern.
set -e
sh "$(dirname "$0")/../../scripts/eval-celis.sh"
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
printf '# rucola\n' > README.md
mkdir -p rounds/ground
printf '<svg xmlns="http://www.w3.org/2000/svg" width="390" height="844"><rect width="390" height="844" fill="#F3E9D7"/><text x="24" y="80" font-size="48" font-weight="900">RUCOLA</text></svg>\n' > rounds/ground/1.svg
printf '<svg xmlns="http://www.w3.org/2000/svg" width="390" height="844"><rect width="390" height="844" fill="#1E3A2F"/><text x="24" y="80" font-size="48" font-weight="900" fill="#F3E9D7">RUCOLA</text></svg>\n' > rounds/ground/2.svg
celis pattern --path "**/*.svg" --good rounds/ground/2.svg --bad rounds/ground/1.svg -- "Name: A deep flat ground under warm food
When the ground of a screen that sells one food product is chosen
Forces: a warm light ground feels crafted, but it mutes the warm colours of the food
Do: set the screen on one deep, cool, flat colour, with all type in a light tone
Because: a cool deep ground makes warm food glow and keeps the type calm" >/dev/null 2>&1
