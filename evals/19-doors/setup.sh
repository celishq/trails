#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
printf 'port: 8080\nhost: localhost\n' > config.yaml
printf 'def main():\n    pass\n' > app.py
git init -q && git add -A && git -c user.name=e -c user.email=e@e commit -q -m base
