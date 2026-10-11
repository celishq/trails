#!/bin/sh
# On each prompt of the person: `celis prompt` writes it on the session's open trail. It prints nothing,
# since the session start and the agent's own `celis read` already show the trail.
input=$(cat)
command -v celis >/dev/null || exit 0
printf '%s' "$input" | celis prompt >/dev/null 2>&1
exit 0
