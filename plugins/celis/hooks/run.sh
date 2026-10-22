#!/bin/sh
# Runs the hook of event $1 (start or ask) for agent $2 (codex, cursor, gemini or antigravity) with `celis
# hook`, or at a start says how to install celis when it is missing.
root="$(cd "$(dirname "$0")/.." && pwd)"
if command -v celis >/dev/null; then
  case "$1" in
    start) exec celis hook start "$2" "$root" ;;
    *) exec celis hook ask "$2" ;;
  esac
fi
[ "$1" = start ] || exit 0
text='The celis plugin needs the celis command, and none is on PATH. Tell the person to install it from https://celis.ai.'
case "$2" in
  cursor) printf '{"additional_context": "%s"}\n' "$text" ;;
  gemini) printf '{"hookSpecificOutput": {"hookEventName": "SessionStart", "additionalContext": "%s"}}\n' "$text" ;;
  antigravity) printf '{"injectSteps": [{"ephemeralMessage": "%s"}]}\n' "$text" ;;
  *) printf '%s\n' "$text" ;;
esac
