#!/bin/sh
# Gives the store the frontend-design skill as a pattern graph, as a learn run drafts it.
set -e
export GIT_CEILING_DIRECTORIES="$(dirname "$PWD")"
unset CLAUDE_CODE_SESSION_ID CODEX_THREAD_ID CELIS_SESSION_ID
d() { celis pattern "$@" 2>/dev/null | head -1; }
page=$(d --path "*.html" --path "*.css" -- "When a page or UI is designed
Forces: a brief leaves axes free, but a model fills free axes with the same defaults
Do: ground each choice in the subject's world, and spend boldness on one element only
Because: a distinct page comes from its subject, and one memorable thing beats many")
d --within "$page" -- "When the order of design work is planned
Forces: building at once is fast, but defaults hide in an unchecked plan
Do: write a token plan of 4 to 6 hex colours, type roles, an ASCII layout and principles; check each part against the brief; then build
Because: a part you would make for any brief is a default, not a choice" >/dev/null
type=$(d --path "*.html" --path "*.css" --within "$page" -- "When the typefaces of a page are chosen
Forces: one family keeps unity, two give contrast, and the usual families read as templated
Do: use one family, or two clearly distinct ones, and not the families you reach for on any project
Because: type carries the personality of the page")
d --path "*.css" --within "$type" -- "When body text is set
Forces: long lines fit more, but the eye loses its place
Do: keep lines under 80 characters, and give serif body a little more line-height than sans
Because: The Elements of Typographic Style asks for it" >/dev/null
d --path "*.html" --path "*.css" --within "$type" -- "When a headline or a label is styled
Forces: an accent draws the eye, but one accented word, all-caps labels and eyebrows are the commonest tells
Do: style the whole headline as one, and use no all-caps label and no eyebrow over content
Because: these treatments mark a generated page" >/dev/null
d --path "*.css" --within "$page" -- "When the palette is chosen and the brief leaves colour free
Forces: cream with terracotta, near-black with one acid accent, and #111 for black are safe, but they appear whatever the subject
Do: take the palette from the subject's materials, and follow the brief when it names one of these looks
Because: a default spent on a free axis is not a choice" >/dev/null
d --path "*.html" --within "$page" -- "When the hero is designed
Forces: a big number with a small label and a gradient is quick, but it is the default treatment
Do: open with the most characteristic thing in the subject's world
Because: the hero is what viewers see first" >/dev/null
d --path "*.html" --path "*.css" --within "$page" -- "When motion that no person starts is added
Forces: motion draws attention, but scattered entrances read as generated
Do: use one orchestrated moment, and else animate only in answer to a person's action
Because: one moment lands better than many" >/dev/null
copy=$(d --path "*.html" --within "$page" -- "When UI copy is written
Forces: clever copy stands out, but people find their way by plain names
Do: name things as users understand them, give a button the verb of what happens, and keep that verb through the flow
Because: the words of an interface are its signposts")
d --path "*.html" --within "$copy" -- "When an error or an empty state is written
Forces: an apology sounds kind, but it hides what went wrong
Do: say what went wrong and how to fix it, with no apology
Because: failure is a moment for direction" >/dev/null
floor=$(d --path "*.html" --path "*.css" --binds --within "$page" -- "When UI is built
Forces: a floor costs time, but a page without it fails people
Do: make it work on a phone, give it a visible keyboard focus, respect reduced motion, and meet contrast
Because: it is the floor, not a choice")
d --path "*.css" --binds --within "$floor" -- "When CSS classes are written
Forces: type and element selectors are quick, but they cancel each other's spacing
Do: check that no two selectors fight over the padding or margin between sections
Because: this often breaks the spacing between sections" >/dev/null
