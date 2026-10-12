#!/bin/sh
# At the start of a session, and after a compaction: says how to install the `celis` command when it is
# missing. Else it prints the trail card and where the rare cases are.
root="$(cd "$(dirname "$0")/.." && pwd)"
if ! command -v celis >/dev/null; then
  echo "The celis plugin needs the \`celis\` command, and none is on PATH. Tell the person to install it with: curl -fsSL https://celis.ai/install.sh | sh"
else
  echo "How to do an ask with the celis plugin:"
  cat "$root/card.md"
  echo
  echo "steps.md: $root/steps.md"
fi
exit 0
