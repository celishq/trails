#!/bin/sh
# Before each tool call, and after each Bash command: prints the rules that `celis rules` gives for the
# call, as Claude Code's context of the call. It prints nothing when `celis` is missing or no rule
# matches, and never stops the call.
root="$(cd "$(dirname "$0")/.." && pwd)"
. "$root/hooks/out.sh"
input=$(cat)
command -v celis >/dev/null || exit 0
dir=$(project "$input")
[ -z "$dir" ] || cd "$dir" || exit 0
rules=$(printf '%s' "$input" | celis rules 2>/dev/null) || exit 0
[ -n "$rules" ] || exit 0
event=$(field "$input" hook_event_name)
case "$event" in
  PostToolUse) lead="celis rules for the files this command changed; check them and fix what breaks one:" ;;
  *) lead="celis rules for this call; keep each one:" ;;
esac
printf '{"hookSpecificOutput": {"hookEventName": "%s", "additionalContext": "%s"}}\n' "$event" "$(json "$lead
$rules")"
exit 0
