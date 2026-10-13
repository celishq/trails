#!/bin/sh
# At the start of a session, and after a compaction: says how to install the `celis` command when it is
# missing. Else it prints the trail card and where the rare cases are. $1 names the agent: claude (the
# default), codex, cursor or gemini.
root="$(cd "$(dirname "$0")/.." && pwd)"
. "$root/hooks/out.sh"
agent=${1:-claude}
input=$(cat)
if ! command -v celis >/dev/null; then
  say "$agent" SessionStart "The celis plugin needs the \`celis\` command, and none is on PATH. Tell the person to install it with: curl -fsSL https://celis.ai/install.sh | sh"
elif [ "$agent" = claude ]; then
  say "$agent" SessionStart "How to do an ask with the celis plugin:
$(cat "$root/card.md")

steps.md: $root/steps.md"
else
  say "$agent" SessionStart "How to do an ask with the celis plugin:
$(cat "$root/card.md")

$(names "$input" "$root")"
fi
exit 0
