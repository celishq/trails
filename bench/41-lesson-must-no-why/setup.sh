#!/bin/sh
set -e
sh "$(dirname "$0")/../../scripts/eval-celis.sh"
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
export CELIS_SESSION_ID=bench
printf '# logs\n' > README.md
id() { head -1 | sed 's/.*"id":"\([0-9a-f]*\)".*/\1/'; }
say() { printf '{"session_id":"bench","prompt":"%s"}' "$1" | celis hook ask claude >/dev/null 2>&1; }
say "Write list_logs(folder) in logs.py: it returns the .log files in the folder, sorted."
ask=$(celis step add --json -- "Write list_logs(folder) in logs.py" 2>/dev/null | id)
a=$(celis step add --under "$ask" --json -- "List the .log files with os.listdir and os.path.join" 2>/dev/null | id)
say "From now on, in all my projects, use pathlib and never os.path."
printf 'from pathlib import Path\n\n\ndef list_logs(folder):\n    return sorted(Path(folder).glob("*.log"))\n' > logs.py
celis step done "$a" --changed "was: os.listdir and os.path.join / now: sorted(Path(folder).glob('*.log'))" --file logs.py -- "list_logs on a folder with a.log, b.txt and c.log gives a.log and c.log" >/dev/null 2>&1
celis step done "$ask" --as-planned -- "list_logs returns the sorted .log files" >/dev/null 2>&1
printf '%s\n' "$ask" > ask.id
