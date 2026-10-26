#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
printf '# greet\n' > README.md
id() { head -1 | sed 's/.*"id":"\([0-9a-f]*\)".*/\1/'; }
ask=$(celis step add --json -- "Write greet(name) in greet.py" | id)
printf 'def greet(name):\n    print("hello " + name)\n    return "hello " + name\n' > greet.py
celis step done "$ask" --as-planned -- "greet('a') gives 'hello a'" >/dev/null 2>&1
