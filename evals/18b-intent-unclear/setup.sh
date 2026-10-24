#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
cat > report.py <<'PY'
def report(sales):
    for s in sales:
        print(s["day"], s["total"])
PY
git init -q && git add -A && git -c user.name=e -c user.email=e@e commit -q -m base
