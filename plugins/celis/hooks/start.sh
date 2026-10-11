#!/bin/sh
# At the start of a session, and after a compaction: says when the `celis` command is missing or older
# than this plugin needs. Else it prints the trail card, and the project's active and recent trails.
NEEDS=$(sed -n 's/.*"version": *"\([^"]*\)".*/\1/p' "$(dirname "$0")/../.claude-plugin/plugin.json")
HAVE=$(celis --version 2>/dev/null | awk '{print $2}')
if [ -z "$HAVE" ]; then
  echo "The celis plugin needs the \`celis\` command $NEEDS or newer, and none is on PATH. Tell the person to run \`/celis:trail setup\`."
elif [ "$(printf '%s\n%s\n' "$NEEDS" "$HAVE" | sort -V | head -1)" != "$NEEDS" ]; then
  echo "The celis plugin needs the \`celis\` command $NEEDS or newer, and PATH has $HAVE. Tell the person to run \`/celis:trail setup\`."
else
  echo "How to do an ask with the celis plugin (\`skills/trail/card.md\`):"
  cat "$(dirname "$0")/../skills/trail/card.md"
  if celis read --json 2>/dev/null | grep -qE '^\{"(trail|recent)"'; then
    echo
    echo "This project has active or recent trails (\`celis read\`). Only when your ask goes on with one, add your first step \`--under <its id>\`."
    celis read 2>&1
  fi
fi
exit 0
