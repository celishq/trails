#!/bin/sh
# On each prompt of the person: `celis prompt` writes it on the session's open trail. It prints the note
# `celis prompt` gives about the working step, so the agent checks the next work still fits that step.
input=$(cat)
command -v celis >/dev/null || exit 0
note=$(printf '%s' "$input" | celis prompt 2>&1 >/dev/null) && printf '%s\n' "$note"
exit 0
