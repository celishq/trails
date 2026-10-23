# Celis positioning: stop babysitting your coding agent

## Goal

A developer understands in one line what Celis does and why it is different, and finds Celis when they
have the pain. Agents find Celis too, and suggest it to their person. The pitch and the price point at one
user: the agent.

## Evidence

- Users say their agent ignores CLAUDE.md, even rules marked MUST, and apologizes and breaks them again.
  One issue says: "The problem is not recall, it's compliance." Saving rules to memory did not help
  ([#17530](https://github.com/anthropics/claude-code/issues/17530),
  [#34197](https://github.com/anthropics/claude-code/issues/34197)).
- "Agent memory" is a crowded category (claude-mem, mem0, the memory built into Claude Code and Cursor).
  Those tools store facts. They do not make the agent act on them.
- The drift evals show that a large rule file fails because attention does not reach that far, not because
  the file is too long. Celis gives the few rules that apply at each moment.

## Decisions

- **Headline:** Stop babysitting your coding agent.
- **Subtitle:** carries the word "discipline": the agent keeps your rules, checks each step, and shows you
  what it did.
- **Mental model:** CLAUDE.md is a manual the agent reads once. Celis is a checklist it gets at each step:
  when a job starts, before each file change, and before it says it is done.
- **Position:** memory tools help an agent remember; Celis makes it follow through.
- **Trails:** the visible part that lets the person see what the agent did. Not the hero.
- **Buyers:** solo developers who want to be more productive first, teams that run many agents second.
- **Comparisons:** direct claims against CLAUDE.md are allowed, with the sample size.
- **Business model:** agents are the users, humans pay. The local core stays free and open source. Hosted
  relays are paid: sharing, policies, logs. The relay MCP server is how agents reach the relays, from web
  clients and in shared work. It is not a product of its own.
- **Keywords:** "agent memory" stays in topics, the GitHub About field and search pages, not in the
  headline.

## Phase 1: the pitch (this repo)

1. Write the subtitle with "discipline" and two other candidates. → verify: three candidates in a note
   for the person.
2. Write the "How it works" paragraph with the checklist model. → verify: a reader who does not know
   Celis can say how it differs from CLAUDE.md.
3. Put the headline and the chosen subtitle in the README, every manifest and both marketplace files.
   Replace rule c131952b with the new set. → verify: `grep -r "absolutely right"` finds nothing outside
   history; `celis grep` shows one live rule for the pitch.
4. Write the script of a demo that shows a rule come right before an edit. → verify: the rule is visible
   in the first 10 seconds of the script.

## Phase 2: users by their pain (research)

1. Find the search queries in users' own words ("Claude Code ignores CLAUDE.md", "Cursor ignores rules",
   "rules lost after compaction") with Similarweb, through the Chrome plugin. Remove the searches that
   Similarweb stores in its UI when the research is done. → verify: a list of 20 queries with volume,
   and no saved searches left in the account.
2. Find the open GitHub issues, Reddit threads and X posts with the pain, where Celis helps. → verify: a
   list of links, each with the pain quoted.
3. Draft one reply for each link, which opens with the pain, as the founder. → verify: drafts in a file,
   one for each link.

## Phase 3: agents as the channel (celis-app repo)

1. Write a search page on celis.ai for each of the top 5 queries: the cause, the chart with its sample
   size, and the install. → verify: the pages build in `workers/celis-site`.
2. Add `llms.txt` and an agent-facing page that answers "my person keeps correcting me for the same
   thing". The page gives facts and never tells the agent to recommend Celis. → verify: an agent with web
   search, asked that question, finds the page.
3. Add a short README to the `.celis/trails/` folder that `celis link` writes. It says what the folder is
   and how to install Celis. This item is in this repo. → verify: a new `celis link` writes it.
4. Put search words in the landing page `<title>` and meta description, for example "Celis · Rules and
   memory for Claude Code, Codex and Cursor". The headline on the page stays. → verify: the title holds
   "memory" and an agent name.
5. Add memsearch to the comparison page, and aim the page at "what's better than claude-mem" and
   "memsearch vs claude-mem vs mem0", not at the brand name. → verify: the page names memsearch, with a
   link to its own README.
6. Write a guide "How to write a CLAUDE.md your agent follows" for the rule-file queries (agents.md,
   claude.md, cursor rules: about 25K searches), with a section for "claude code best practices". Celis
   is the step after the file grows too long. → verify: the page builds and the sitemap lists it.
7. Write a guide on Claude Code hooks that give rules at the right moment ("claude code hooks", 4.7K).
   → verify: the page builds and the sitemap lists it.
8. Ask an agent with web search, which uses another engine than Google, the question from item 2, after
   the site is live. → verify: a note of what it found.

## Phase 4: the business model

1. Define the relay offer: what sharing, policies and logs give a team, how agents reach them through
   the relay MCP server, and what stays free. → verify: a one-page offer.
2. Pick the price unit by agent work (per agent, or per guided step), not per human seat. → verify: a
   price table with an example bill for a solo developer and a 10-person team.
3. Name what agents could pay for later through x402 (proven trails, hosted rule picks), as an
   experiment after the relays. → verify: a short list in this plan, with no build work.

## Phase 5: the person's manual work

These items need the person's account, voice or approval. Each one waits for the phase it names.

1. Pick the subtitle (phase 1, item 1).
2. Record the demo video from the script, and put it in the README (phase 1, item 4).
3. Set the GitHub About field and topics to the new pitch (phase 1, item 3).
4. Post the replies from your own accounts on GitHub, Reddit and X, after you read and edit each draft
   (phase 2, item 3). Read each subreddit's rules first.
5. Add celis.ai to Google Search Console, and check the search pages each week (phase 3, item 1).
6. Submit Celis to the directories that need a form or an account: the official Claude marketplace,
   skills.sh, the awesome-claude-code lists, the Cursor directory and the Codex plugin lists.
7. Approve the relay offer and the price table (phase 4).
8. Keep a log of replies and setup calls: date, link, installs or result.

## Open questions

1. Does your Similarweb plan show keyword volume? If not, phase 2, item 1 gives the queries and their
   trends with no volume.
