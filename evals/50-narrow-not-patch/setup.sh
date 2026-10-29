#!/bin/sh
# Gives the store a pattern for one product screen, and a done landing page that followed it too far.
set -e
sh "$(dirname "$0")/../../scripts/eval-celis.sh"
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
id() { head -1 | sed 's/.*"id":"\([0-9a-f]*\)".*/\1/'; }
printf '# rucola\n' > README.md
printf '# Rucola\n\nMenu: Margherita 9 EUR, Funghi 11 EUR, Diavola 12 EUR.\nDelivery: Mitte and Kreuzberg, free over 20 EUR.\nHours: 12:00 to 23:00 every day.\n' > facts.md
big=$(celis pattern --json --path "**/*.html" -- "Name: One big product over one bold word
When the layout of a food app screen is made
Forces: a menu list of many small items shows more choice, but each item is small and the screen reads busy
Do: show one product big, over one bold word, with little else on the screen
Because: a single large product and bold type give one clear thing to want" | id)
ask=$(celis step add --uses "$big" --json -- "Landing page for Rucola, a pizza delivery app" | id)
printf '<!doctype html>\n<meta name="viewport" content="width=device-width">\n<style>body{margin:0;background:#1E3A2F;color:#F3E9D7;font-family:sans-serif}section{min-height:100vh;display:flex;flex-direction:column;justify-content:space-between;padding:24px}h1{font-size:72px;font-weight:900;margin:0}</style>\n<section><h1>Rucola</h1><div style="font-size:160px">&#127829;</div><p>MARGHERITA &middot; 9 EUR &middot; <a href="#">ORDER</a></p></section>\n' > index.html
celis step done "$ask" --as-planned --file index.html -- "index.html shows one screen: the name, one big pizza, and price and order at the bottom" >/dev/null 2>&1
