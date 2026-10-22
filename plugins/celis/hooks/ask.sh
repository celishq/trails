#!/bin/sh
# On each Claude Code prompt: records the prompt on the session's open trail, and prints the note celis gives.
command -v celis >/dev/null || exit 0
celis hook ask claude || exit 0
