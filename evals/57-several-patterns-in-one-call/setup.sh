#!/bin/sh
set -e
sh "$(dirname "$0")/../../scripts/eval-celis.sh"
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
printf 'def parse_quantity(text):\n    return int(text)\n' > order.py
git init -q
git add order.py
git -c user.name=eval -c user.email=eval@example.com commit -qm "first"
celis pattern -- "Name: Error messages that name the value and the rule
When an input check refuses a value
Forces: a short message is quick to write, but the reader cannot tell what was wrong
Do: name the value given and the rule it broke in the message
Because: the reader fixes the input without reading the code" >/dev/null 2>&1
celis pattern -- "Name: One test for each side of a boundary
When a check gets a boundary, such as a lowest allowed number
Forces: one test is quick, but a check off by one passes it
Do: test the value on the boundary and the value just past it
Because: both sides together catch an off-by-one" >/dev/null 2>&1
celis pattern -- "Name: Test names that say the rule
When a unittest test is added
Forces: a short name is easy to type, but a failing run then says nothing
Do: name the test after the rule it proves, such as test_refuses_zero
Because: a failing run then says which rule broke" >/dev/null 2>&1
celis pattern -- "Name: Prices in whole cents
When a price is stored or summed
Forces: a float is easy to use, but sums of floats round
Do: keep each price as a whole number of cents
Because: a sum of whole numbers never rounds" >/dev/null 2>&1
