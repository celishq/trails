#!/bin/sh
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
mkdir -p shop
printf 'import logging\n\nlog = logging.getLogger("shop")\n\n\ndef price(cents):\n    return cents\n' > shop/prices.py
git init -q && git add -A && git -c user.name=e -c user.email=e@e commit -q -m base
celis pattern --scope me --path "shop/*.py" -- "When shop code changes a price: log the old and the new price with log.info, never print, because the ops team audits price changes from the logs." >/dev/null
celis pattern --scope me --path "docs/**" -- "When you write docs: keep each page under one screen, because people skim." >/dev/null
celis pattern --scope me --path "**/*.sql" -- "When you add a migration: never edit an applied one, because the servers already ran it." >/dev/null
