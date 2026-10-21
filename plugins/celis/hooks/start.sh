#!/bin/sh
# At the start of a session, and after a compaction: says how to install the `celis` command when it is
# missing. Else it prints the trail card, where the rare cases are, the named trails most used, and a notice when
# `celis` and the plugin have different versions. After a compaction it lets `celis prompt` ask for the past agent again. $1 names
# the agent: claude (the default), codex, cursor, gemini or antigravity. Antigravity has no session start,
# and runs this before each call of the model.
root="$(cd "$(dirname "$0")/.." && pwd)"
. "$root/hooks/out.sh"
agent=${1:-claude}
input=$(cat)
command -v celis >/dev/null && printf '%s' "$input" | celis prompt >/dev/null 2>&1
installed=$(celis --version 2>/dev/null | cut -d ' ' -f 2)
plugin=$(field "$(cat "$root/.claude-plugin/plugin.json")" version)
named=$(celis names --limit 20 2>/dev/null)
[ -z "$named" ] || named="

Named trails: when an ask does what one does, add the ask with --uses <its name>.
$named"
notice=""
case "$installed:$plugin" in
  0.0.0:* | *:0.0.0 | :* | *: | "$plugin:$installed") ;;
  *) notice="

celis $installed is installed, and the celis plugin is $plugin. Tell the person to update the older one: the command with \`celis update\`, the plugin with their agent's plugin update." ;;
esac
if ! command -v celis >/dev/null; then
  say "$agent" SessionStart "The celis plugin needs the \`celis\` command, and none is on PATH. Tell the person to install it with: curl -fsSL https://celis.ai/install.sh | sh"
elif [ "$agent" = claude ]; then
  say "$agent" SessionStart "How to do an ask with the celis plugin:
$(cat "$root/card.md")

steps.md: $root/steps.md$named$notice"
else
  say "$agent" SessionStart "How to do an ask with the celis plugin:
$(cat "$root/card.md")

$(names "$input" "$root")$named$notice"
fi
exit 0
