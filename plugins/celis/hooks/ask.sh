#!/bin/sh
# On each prompt of the person: `celis prompt` writes it on the session's open trail, and this prints the
# note it gives about the working step. With no open trail, it prints the past work like the prompt.
input=$(cat)
command -v celis >/dev/null || exit 0
note=$(printf '%s' "$input" | celis prompt 2>&1 >/dev/null) && printf '%s\n' "$note"
exit 0
