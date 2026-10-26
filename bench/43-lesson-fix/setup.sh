#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
export CELIS_SESSION_ID=bench
printf '# orders\n' > README.md
id() { head -1 | sed 's/.*"id":"\([0-9a-f]*\)".*/\1/'; }
say() { printf '{"session_id":"bench","prompt":"%s"}' "$1" | celis hook ask claude >/dev/null 2>&1; }
say "Add a created_at field to each order that export.py writes for the warehouse."
first=$(celis step add --json -- "Add created_at to the orders export.py writes" 2>/dev/null | id)
printf 'from datetime import datetime\n\n\ndef export(order):\n    return {**order, "created_at": datetime.now().isoformat()}\n' > export.py
celis step done "$first" --as-planned --file export.py -- "export({}) holds created_at" >/dev/null 2>&1
say "The export dates are wrong for the Berlin warehouse, its orders come out an hour off."
ask=$(celis step add --replaces "$first" --json -- "Fix the created_at of the export for the Berlin warehouse" 2>/dev/null | id)
printf 'from datetime import datetime, timezone\n\n\ndef export(order):\n    return {**order, "created_at": datetime.now(timezone.utc).isoformat()}\n' > export.py
celis step done "$ask" --changed "was: datetime.now().isoformat(), the server's local time with no offset / now: datetime.now(timezone.utc).isoformat(), UTC with its +00:00 offset" --file export.py -- "export({}) gives created_at ending in +00:00" >/dev/null 2>&1
printf '%s\n' "$ask" > ask.id
