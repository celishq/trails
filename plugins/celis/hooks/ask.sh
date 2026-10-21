#!/bin/sh
# On each prompt: writes the prompt on the session's open trail, and prints the note celis gives the agent.
# $1 names the agent: claude (the default), codex, cursor or gemini.
root="$(cd "$(dirname "$0")/.." && pwd)"
. "$root/hooks/out.sh"
agent=${1:-claude}
input=$(cat)
command -v celis >/dev/null || exit 0
dir=$(project "$input")
[ -z "$dir" ] || cd "$dir" || exit 0
note=$(printf '%s' "$input" | celis prompt 2>&1 >/dev/null) || note=
[ "$agent" = codex ] || [ "$agent" = gemini ] && note=$(printf '%s' "$note" | sed 's/run the celis:past agent in the foreground with the ask and the places it may touch, and wait for its answer/do what past.md says, yourself, for the ask and the places it may touch/')
case "$agent" in
  cursor) echo '{"continue": true}' ;;
  *) [ -n "$note" ] && say "$agent" BeforeAgent "$note" ;;
esac
exit 0
