#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
id() { head -1 | sed 's/.*"id":"\([0-9a-f]*\)".*/\1/'; }
trail=$(celis step add --new --json -- "Write the shop blurbs" | id)
a=$(celis step add --under "$trail" --json -- "Write the blurb of the coffee page in coffee.md" | id)
printf 'Fresh beans. Roasted Monday. Gone by Friday.\n' > coffee.md
celis step done "$a" --as-planned --file coffee.md -- "coffee.md holds the blurb" >/dev/null 2>&1
b=$(celis step add --under "$trail" --json -- "Write the blurb of the tea page in tea.md" | id)
printf 'Our carefully curated selection of exquisite teas, sourced from the finest gardens across the world, offers a truly unforgettable experience for every discerning palate.\n' > tea.md
celis step done "$b" --as-planned --file tea.md -- "tea.md holds the blurb" >/dev/null 2>&1
