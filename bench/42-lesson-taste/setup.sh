#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
export CELIS_SESSION_ID=bench
printf '# photos\n' > README.md
id() { head -1 | sed 's/.*"id":"\([0-9a-f]*\)".*/\1/'; }
say() { printf '{"session_id":"bench","prompt":"%s"}' "$1" | celis hook ask claude >/dev/null 2>&1; }
say "Write two versions of the error message for an upload that is too large, so I can pick one."
ask=$(celis step add --json -- "Write two versions of the error message for a too large upload" 2>/dev/null | id)
a=$(celis step add --under "$ask" --json -- "Write version A of the message in msg-a.txt" 2>/dev/null | id)
printf "Oops! Something went wrong while we were trying to upload your file. Please don't worry, these things happen! Maybe try again in a little while?\n" > msg-a.txt
celis step done "$a" --as-planned --file msg-a.txt -- "msg-a.txt holds version A" >/dev/null 2>&1
b=$(celis step add --under "$ask" --json -- "Write version B of the message in msg-b.txt" 2>/dev/null | id)
printf 'Upload failed: photo.png is over the 10 MB limit. Make it smaller and try again.\n' > msg-b.txt
celis step done "$b" --as-planned --file msg-b.txt -- "msg-b.txt holds version B" >/dev/null 2>&1
say "B. A is too chatty."
celis step done "$ask" --by person --changed "the person picked version B, msg-b.txt, over version A, msg-a.txt" -- "the person picked B" >/dev/null 2>&1
printf '%s\n' "$ask" > ask.id
