#!/bin/sh
# At the start of a session, and after a compaction: says how to install the `celis` command when it is
# missing. Else it prints the trail card and where the rare cases are, and after a compaction lets
# `celis prompt` ask for the past agent again. $1 names the agent: claude (the default), codex, cursor,
# gemini or antigravity. Antigravity has no session start, and runs this before each call of the model.
root="$(cd "$(dirname "$0")/.." && pwd)"
. "$root/hooks/out.sh"
agent=${1:-claude}
input=$(cat)
command -v celis >/dev/null && printf '%s' "$input" | celis prompt >/dev/null 2>&1
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
