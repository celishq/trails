#!/bin/sh
# Says, at the start of a session, when the `celis` command is missing or older than this plugin needs.
# It prints nothing when `celis` is fine.
NEEDS=0.11.0
HAVE=$(celis --version 2>/dev/null | awk '{print $2}')
if [ -z "$HAVE" ]; then
  echo "The celis plugin needs the \`celis\` command $NEEDS or newer, and none is on PATH. Tell the person to run \`/celis:trail setup\`."
elif [ "$(printf '%s\n%s\n' "$NEEDS" "$HAVE" | sort -V | head -1)" != "$NEEDS" ]; then
  echo "The celis plugin needs the \`celis\` command $NEEDS or newer, and PATH has $HAVE. Tell the person to run \`/celis:trail setup\`."
fi
