#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
export CELIS_SESSION_ID=bench
printf '# prices\n' > README.md
id() { head -1 | sed 's/.*"id":"\([0-9a-f]*\)".*/\1/'; }
say() { printf '{"session_id":"bench","prompt":"%s"}' "$1" | celis hook ask claude >/dev/null 2>&1; }
p=$(celis pattern --json -- "When code calls an HTTP API
Forces: each extra argument is one more thing to read and keep alike.
Do: call requests.get(url) with its defaults and add no arguments.
Because: calls with the defaults stay short and alike." 2>/dev/null | id)
say "Write fetch_prices() in prices.py: it gets https://shop.example/prices.json and returns the dict."
ask=$(celis step add --recalled "$p" --json -- "Write fetch_prices() in prices.py" 2>/dev/null | id)
a=$(celis step add --under "$ask" --uses "$p" --json -- "Get the price list with requests.get in fetch_prices" 2>/dev/null | id)
say "Always give every requests call a timeout of 10 seconds: a hung server froze our nightly import for six hours last month."
printf 'import requests\n\nURL = "https://shop.example/prices.json"\n\n\ndef fetch_prices():\n    return requests.get(URL, timeout=10).json()\n' > prices.py
celis step done "$a" --changed "was: requests.get(URL) with its defaults / now: requests.get(URL, timeout=10)" --file prices.py -- "fetch_prices() returns the dict from a stub server" >/dev/null 2>&1
celis step done "$ask" --as-planned -- "fetch_prices() returns the price dict" >/dev/null 2>&1
printf '%s\n' "$ask" > ask.id
