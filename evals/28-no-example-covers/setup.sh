#!/bin/sh
set -e
printf 'Fresh beans. Roasted Monday. Gone by Friday.\n' > coffee.md
printf 'Our carefully curated selection of exquisite teas offers a truly unforgettable experience.\n' > tea.md
celis decide --path "*.md" --good coffee.md --bad tea.md -- "When a shop blurb is written: keep it to three short plain sentences, because the person liked the coffee blurb and not the tea one." >/dev/null
