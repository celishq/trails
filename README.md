# celis trails

```
                   /\                                 /\
          /\      /::\         /\        /\          /::\
         /::\    /::::\       /::\      /::\    /\  /::::\        |>
        /::::\  /::::::\     /::::\    /::::\  /::\/::::::\       |
  _____/::::::\/::::::::\___/::::::\__/::::::\/:::/::::::::\   .--+--.
                                                              /+++++++\
         ______   ______   <-- this sign was left            /+++++++++\
        /  x   | |  |>  \      by the agents before you     /+++++++++++\
        \______| |______/             |>                 .-'+++++++++++++'-.
               | |                   o|                 /+++++++++++++++++++\
  .-.   .-.    | |                   ▮                 /+++++++++++++++++++++\
 |RIP| |RIP| . |_| ................ / \.............../+++++++++++++++++++++++\
 ,      ,      ,             ,             ,             ,             ,
```

Celis is an executable binary and a plugin for AI agents that gives them "experience". During the work agents write down
what worked, what didn't and why, so the next agent takes the good roads and avoids the traps.

## How it works

Your agent doesn't plan the whole job up front. It finds its way one step at a time, like a map app that
tries the most promising road first.

```
Task: seniors pay 20% less
├─ ✓ Find where prices are set
├─ ◐ Change the price rule                     ← least sure about this part: break it down
│  ├─ ✗ Change the price in the cart              gave up: the cart doesn't know the age
│  ├─ ✓ Write a test: a senior pays 80%           proof: it failed before, now it passes
│  │                                              lesson: tests need SHELF_OFFLINE=1
│  └─ ○ Use the rule in prices.py              ← the next step, and only the next one
└─ ○ Show the senior price on the page         ← not broken down yet

✓ done   ◐ working on it   ○ to do   ✗ dropped
```

1. **Start with the big picture.** The agent splits the task into a few big parts. It picks the part it's
   least sure about, or the one the others depend on, and breaks only that part into smaller pieces.
2. **Take one step at a time.** The agent does one small step and checks that it worked as expected. Only
   then does it pick the next one. If a road leads nowhere, it turns back and tries another.
3. **Show proof.** A step counts as done only with proof you can check, like a test that now passes.
4. **Write down the lessons.** When a road fails, the agent notes why. When something wastes time, it
   notes the trap. The next agent that works on the same file reads these notes first.
5. **Reuse what worked.** celis remembers how each task went: how long it took, and how often you had to
   correct the agent. Before a new task, the agent looks at similar past tasks from all your projects and
   follows the roads that went best.

```
New task: students pay 10% less

Similar past tasks:
  ✓ shop   seniors pay 20% less     40 min · corrected 0 times   ← follow this road
  ✓ cafe   happy hour prices        3 h    · corrected 4 times

Lessons for prices.py:
  • tests need SHELF_OFFLINE=1
  • the cart doesn't know the age: change the price in prices.py
```

Everything is saved as [nostr](https://nostr.com) notes, signed with your own key.

## Install

**1. Install `celis`:**

```bash
curl -fsSL https://celis.ai/install.sh | sh
```

**2. Add the plugin to your agent:**

| Agent       | Install                                                                         | Then                                                                              |
| ----------- | ------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| Claude Code | `/plugin marketplace add celishq/trails`<br>`/plugin install celis@celis`       | —                                                                                 |
| Codex       | `codex plugin marketplace add celishq/trails`<br>`codex plugin add celis@celis` | Trust its hooks with `/hooks`.                                                    |
| Cursor      | `cursor-agent plugin marketplace add https://github.com/celishq/trails`         | Install `celis` from `/plugins`. Cursor doesn't remind the agent on each message. |
| Gemini CLI  | `gemini extensions install https://github.com/celishq/trails`                   | —                                                                                 |

**3. Give your agent a task.** When the task changes files, the agent keeps its steps in celis. Run
`celis tree` in a second terminal to watch them as the agent works.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
