# What the hooks source: prints a hook's text in the shape the agent reads, and the lines that
# name the session and the plugin's files to an agent other than Claude Code.

# The text $3 for the agent $1 on the hook event $2.
say() {
  case "$1" in
    cursor) printf '{"additional_context": "%s"}\n' "$(json "$3")" ;;
    gemini) printf '{"hookSpecificOutput": {"hookEventName": "%s", "additionalContext": "%s"}}\n' "$2" "$(json "$3")" ;;
    antigravity) printf '{"injectSteps": [{"ephemeralMessage": "%s"}]}\n' "$(json "$3")" ;;
    *) printf '%s\n' "$3" ;;
  esac
}

# The text $1 as the inside of a JSON string.
json() {
  printf '%s' "$1" | awk 'BEGIN { ORS = "" } { gsub(/\\/, "\\\\"); gsub(/"/, "\\\""); gsub(/\t/, "\\t"); if (NR > 1) print "\\n"; print }'
}

# The string field $2 of the hook's JSON $1; with `[`, the first string of the list field $2.
field() {
  printf '%s' "$1" | sed -n "s/.*\"$2\" *: *$3\"\([^\"]*\)\".*/\1/p" | head -n 1
}

# The person's project of the hook's JSON $1: its `cwd`, else Cursor's first workspace root.
project() {
  dir=$(field "$1" cwd)
  [ -n "$dir" ] || dir=$(field "$1" workspace_roots '\[ *')
  printf '%s' "$dir"
}

# The session of the hook's JSON $1, and where steps.md and past.md are under the plugin root $2.
names() {
  id=$(field "$1" conversation_id)
  [ -n "$id" ] || id=$(field "$1" conversationId)
  [ -n "$id" ] || id=$(field "$1" session_id)
  echo "celis session: $id. Pass --session $id to each celis step add."
  echo "steps.md: $2/steps.md"
  echo "past.md: $2/agents/past.md"
}
