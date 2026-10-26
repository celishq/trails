#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
printf 'Fresh beans. Roasted Monday. Gone by Friday.\n' > coffee.md
printf 'Our carefully curated selection of exquisite teas offers a truly unforgettable experience.\n' > tea.md
celis pattern --path "*.md" --good coffee.md --bad tea.md -- "When a shop blurb is written: keep it to three short plain sentences, because the person liked the coffee blurb and not the tea one." >/dev/null
