#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
mkdir -p shop migrations
printf 'def total(prices):\n    return sum(prices)\n' > shop/prices.py
printf 'CREATE TABLE price (cents INTEGER);\n' > migrations/0001_price.sql
celis decide -- "When code handles money: keep amounts as integer cents, because floats lose a cent." >/dev/null
celis decide --scope me -- "When you write a git commit message: use the imperative mood, because the team reads the log that way." >/dev/null
celis decide -- "When the person asks for a plan: split it into phases that one agent can take alone, because they hand phases to other agents." >/dev/null
celis decide -- "The staging server runs on port 8123 this week." >/dev/null
celis decide --path "migrations/*.sql" -- "When you add a migration: never edit an applied one, because the servers already ran it." >/dev/null
trail=$(celis step add --new -- "Store prices in shop/prices.py" 2>/dev/null)
step=$(celis step add --under "$trail" -- "Keep each price as a float in shop/prices.py" 2>/dev/null)
celis decide --scope "trail:$trail" -- "When you store a price in shop/prices.py: keep it as a float, because the first shop did." >/dev/null
celis step done "$step" --as-planned -- "shop/prices.py stores floats." >/dev/null 2>&1
celis step done "$trail" --as-planned -- "shop/prices.py stores floats." >/dev/null 2>&1
fix=$(celis step add --new -- "Fix the rounding of prices" 2>/dev/null)
cents=$(celis step add --under "$fix" --replaces "$step" -- "Keep each price as integer cents in shop/prices.py" 2>/dev/null)
celis step done "$cents" --as-planned -- "shop/prices.py stores cents." >/dev/null 2>&1
celis step done "$fix" --as-planned -- "shop/prices.py stores cents." >/dev/null 2>&1
