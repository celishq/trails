#!/bin/sh
# On each prompt of the person: `celis prompt` writes it on the session's open trail. For a
# `/celis:trail` ask, it then prints `celis read`, so each ask starts with the trail. It prints nothing
# else, or when `celis` is missing.
input=$(cat)
command -v celis >/dev/null || exit 0
printf '%s' "$input" | celis prompt >/dev/null 2>&1
if printf '%s' "$input" | grep -q '"prompt" *: *"[[:space:]]*/celis:trail'; then
  trail=$(celis read 2>&1)
  echo "The trail at the start of this ask (\`celis read\`):"
  echo "${trail:-nothing yet}"
fi
exit 0
