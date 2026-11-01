#!/bin/sh
set -e
sh "$(dirname "$0")/../../scripts/eval-celis.sh"
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
mkdir -p style
printf 'Fresh beans. Roasted Monday. Gone by Friday.\n' > style/blurb-good.md
printf 'Our carefully curated selection offers a truly unforgettable experience.\n' > style/blurb-bad.md
printf 'the age is negative: give 0 or more\n' > style/error-good.txt
printf 'invalid input\n' > style/error-bad.txt
printf 'fix: keep cart totals in cents\n' > style/commit-good.txt
printf 'Updated some stuff in the cart and fixed things\n' > style/commit-bad.txt
printf '# Buy\n' > style/button-good.md
printf '# Click here to proceed with your purchase now!\n' > style/button-bad.md
printf '| cake | price |\n|---|---|\n| lemon | 4 |\n' > style/table-good.md
printf 'Lemon cake costs 4, and the plum cake, which is new, costs 5.\n' > style/table-bad.md
cat > STYLE.md <<'MD'
# How I like things

1. Blurbs: short and plain, like style/blurb-good.md, not like style/blurb-bad.md.
2. Error messages: say what to give instead, like style/error-good.txt, not style/error-bad.txt.
3. Commit messages: a type and one plain line, like style/commit-good.txt, not style/commit-bad.txt.
4. Buttons: one verb, like style/button-good.md, not style/button-bad.md.
5. Prices: in a table, like style/table-good.md, not in prose like style/table-bad.md.
MD
