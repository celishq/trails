# What start.sh and ask.sh source: prints a hook's text in the shape the agent reads, and the lines that
# name the session and the plugin's files to an agent other than Claude Code.

# The text $3 for the agent $1 on the hook event $2.
say() {
  case "$1" in
    cursor) printf '{"additional_context": "%s"}\n' "$(json "$3")" ;;
    gemini) printf '{"hookSpecificOutput": {"hookEventName": "%s", "additionalContext": "%s"}}\n' "$2" "$(json "$3")" ;;
    *) printf '%s\n' "$3" ;;
  esac
}

# The text $1 as the inside of a JSON string.
json() {
  printf '%s' "$1" | awk 'BEGIN { ORS = "" } { gsub(/\\/, "\\\\"); gsub(/"/, "\\\""); gsub(/\t/, "\\t"); if (NR > 1) print "\\n"; print }'
}

# The session of the hook's JSON $1, and where steps.md and past.md are under the plugin root $2.
names() {
  id=$(printf '%s' "$1" | sed -n 's/.*"conversation_id" *: *"\([^"]*\)".*/\1/p' | head -n 1)
  [ -n "$id" ] || id=$(printf '%s' "$1" | sed -n 's/.*"session_id" *: *"\([^"]*\)".*/\1/p' | head -n 1)
  echo "celis session: $id. Pass --session $id to each celis step add."
  echo "steps.md: $2/steps.md"
  echo "past.md: $2/agents/past.md"
}
