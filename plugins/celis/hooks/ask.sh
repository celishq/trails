#!/bin/sh
# On each prompt of the person: `celis prompt` writes it on the session's open trail, and this prints the
# note it gives about the working step. With no open trail, it names the agent that finds past work. $1
# names the agent: claude (the default), codex, cursor, gemini or kimi.
root="$(cd "$(dirname "$0")/.." && pwd)"
. "$root/hooks/out.sh"
agent=${1:-claude}
input=$(cat)
command -v celis >/dev/null || exit 0
note=$(printf '%s' "$input" | celis prompt 2>&1 >/dev/null) || note=
[ "$agent" = kimi ] && note="$(names "$input" "$root")${note:+
$note}"
case "$agent" in
  cursor) echo '{"continue": true}' ;;
  *) [ -n "$note" ] && say "$agent" BeforeAgent "$note" ;;
esac
exit 0
