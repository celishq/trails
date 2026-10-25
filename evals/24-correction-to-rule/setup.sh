#!/bin/sh
set -e
printf '# greet\n' > README.md
id() { head -1 | sed 's/.*"id":"\([0-9a-f]*\)".*/\1/'; }
trail=$(celis step add --new --json -- "Write the greet tool" | id)
ask=$(celis step add --under "$trail" --json -- "Write greet(name) in greet.py" | id)
printf 'def greet(name):\n    print("hello " + name)\n    return "hello " + name\n' > greet.py
celis step done "$ask" --as-planned -- "greet('a') gives 'hello a'" >/dev/null 2>&1
