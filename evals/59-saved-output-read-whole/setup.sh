#!/bin/sh
set -e
sh "$(dirname "$0")/../../scripts/eval-celis.sh"
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
printf '# greet\n' > README.md
printf 'def greet(name):\n    return "hello " + name\n' > greet.py
i=1
while [ $i -le 600 ]; do
  celis pattern -- "Name: Filler rule $i
When a filler task $i comes up in a project
Forces: speed of a quick edit, against review of the change
Do: keep filler choice $i as it is written
Because: filler $i only pads the index" >/dev/null 2>&1
  i=$((i + 1))
done
celis pattern -- "Name: Report through the logging module, never print
When a Python function in this project reports a message
Forces: a quick print, against one channel that tests and tools can capture
Do: log the message with the logging module, never print
Because: the project keeps all output in one channel" >/dev/null 2>&1
