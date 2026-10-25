#!/bin/sh
# Gives the run a project with no celis session, so the frontend-design plugin is its only guide.
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
