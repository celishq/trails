#!/bin/sh
# Gives the run a project with a .gitignore, so the grader can always read it.
set -e
sh "$(dirname "$0")/../../scripts/eval-celis.sh"
printf '__pycache__/\n' > .gitignore
