---
type: llm
focus: trace
arm: with-only
---
Read each `celis pattern` command in the trace and the final reply. Pass only if all of these hold:

- The patterns hold only the lesson that each requests call gets a timeout: no pattern about fetch_prices, the price URL or the dict it returns.
- Each pattern has four lines, When, Forces, Do and Because, and its When names the kind of work, never "this project", the repo or the person.
- Each Because gives a reason the choice is good for the work, such as what goes wrong without it; a Because that says only that the person asked, wants or prefers it fails; here the person gave the reason, a hung server that froze the nightly import for six hours, so a Because that says a call with no timeout can hang the work passes.
- The brief lists a pattern in play, `pattern <id8> When code calls an HTTP API`, which said to call requests.get with its defaults; the command that writes the timeout pattern has `--replaces` with that id or its first 4 or more characters.
