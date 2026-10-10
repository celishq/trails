#!/bin/sh
# Prints `celis read` when the person's prompt is a `/celis:trail` ask, so each ask starts with the
# trail. For any other prompt while a trail of this project is active, it reminds the agent to record the
# prompt as a steer. It prints nothing else, or when `celis` is missing.
prompt=$(cat)
command -v celis >/dev/null || exit 0
if printf '%s' "$prompt" | grep -q '"prompt" *: *"/celis:trail'; then
  trail=$(celis read 2>&1)
  echo "The trail at the start of this ask (\`celis read\`):"
  echo "${trail:-nothing yet}"
elif celis read --json 2>/dev/null | grep -q '^{"trail"'; then
  echo "A trail is active. If this prompt changes the work on an open step, record it first: \`celis step steer <id> -- \"<what the person asked, in one sentence>\"\`."
fi
exit 0
