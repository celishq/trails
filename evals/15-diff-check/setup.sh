#!/bin/sh
set -e
sh "$(dirname "$0")/../../scripts/eval-celis.sh"
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
printf '# tools\n' > README.md
git init -q && git add -A && git -c user.name=e -c user.email=e@e commit -q -m base
celis pattern --scope me --path "**/*.py" -- "When a Python file changes: give each public function a docstring and type hints, because the docs are built from them." >/dev/null
celis pattern --scope me --path "**/*.py" -- "When a module is added: list it in README.md with one line on what it does, because new people start from the README." >/dev/null
