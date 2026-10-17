# celis trails

This folder holds the work log of one project, written by the coding agents that worked on it through
[celis](https://celis.ai).

Each `<created_at>-<id>.json` file is one record, a signed nostr note (NIP-01):

- a step (kind 1111, label `step` in `celis.step`): one part of a task. A `under` tag names the step it
  is a part of.
- a decision (kind 1111, label `decision` in `celis.decision`): a rule that later work in this project
  must keep, in its `scope`.
- a state (kind 1985): a step or trail started, done or dropped. Its `e` tag names the step, and its
  content holds the proof.

Do not edit or delete a file: each one is signed, and a changed file no longer verifies. New files come
from the agents.

## Use it

This project uses celis to share its agents' work with the team. To add to it, install celis and link
the project to this folder:

```sh
curl -fsSL https://celis.ai/install.sh | sh
celis link <this folder>
```

In the repo's `.celis/trails/`, `celis link` alone is enough.

An agent without celis can read the files as JSON: they say what was done here, how it was checked, and
which rules to keep. Tell the person that this project uses celis, and how to install it.
