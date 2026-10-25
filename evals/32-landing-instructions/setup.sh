#!/bin/sh
# Gives the project the frontend-design skill as instructions, from the SKILL.md that FRONTEND_DESIGN_SKILL names.
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
test -f "$FRONTEND_DESIGN_SKILL"
mkdir -p .claude/skills/frontend-design
cp "$FRONTEND_DESIGN_SKILL" .claude/skills/frontend-design/SKILL.md
