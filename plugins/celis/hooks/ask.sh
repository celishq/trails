#!/bin/sh
# On each prompt of the person: `celis prompt` writes it on the session's open trail, and this prints the
# note it gives about the working step. With no open trail, it names the agent that finds past work. $1
# names the agent: claude (the default), codex, cursor or gemini.
root="$(cd "$(dirname "$0")/.." && pwd)"
. "$root/hooks/out.sh"
agent=${1:-claude}
input=$(cat)
command -v celis >/dev/null || exit 0
dir=$(project "$input")
[ -z "$dir" ] || cd "$dir" || exit 0
note=$(printf '%s' "$input" | celis prompt 2>&1 >/dev/null) || note=
case "$agent" in
  cursor) echo '{"continue": true}' ;;
  *) [ -n "$note" ] && say "$agent" BeforeAgent "$note" ;;
esac
exit 0
