#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
printf 'def client_count(clients):\n    return len(clients)\n' > report.py
larger=$(celis pattern -- "When data leaves the app for another system: quote every field, such as with csv.QUOTE_ALL, because the bank's importer read client ids as numbers and dropped their leading zeros." | head -n 1)
celis pattern --path report.py --within "$larger" -- "When a CSV report is added to report.py: write it with csv.writer and a header row of the field names, because a plain join broke on a name with a comma." >/dev/null
