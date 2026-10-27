#!/bin/sh
set -e
sh "$(dirname "$0")/../../scripts/eval-celis.sh"
cp "$(dirname "$0")/../answer.md" CLAUDE.md
