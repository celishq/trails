#!/bin/sh
# Gives the run a project with a .gitignore, so the grader can always read it.
set -e
printf '__pycache__/\n' > .gitignore
