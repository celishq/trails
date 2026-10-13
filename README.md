# Celis Trails

```
                   /\                                 /\
          /\      /::\         /\        /\          /::\
         /::\    /::::\       /::\      /::\    /\  /::::\
        /::::\  /::::::\     /::::\    /::::\  /::\/::::::\      |>
  _____/::::::\/::::::::\___/::::::\__/::::::\/:::/::::::::\     |
                                                              .--+--.
         ______   ______                                     /+++++++\
        /  x   | |  |>  \                                   /+++++++++\
        \______| |______/                                  /+++++++++++\
      dead end | | next step                            .-'+++++++++++++'-.
               |/|/                                     /+++++++++++++++++\
   x_x                                                          //
  |RIP|  x_x                       •_•|>                       //
 .......|RIP|.......................O.|.......................//...............
 ,      ,      ,      ,      ,      ,      ,      ,      ,      ,      ,      ,
```

Celis is an executable binary and a plugin for AI agents. It modifies the agents’ behavior so they begin to track what worked, what didn’t, and why. This allows the next agent to avoid repeating mistakes, complete tasks faster at a lower cost.

## How it works

Once Celis is installed, the agent stops planning the entire task in advance. It finds the nearest “small step” and checks whether that step brings it closer to its goal or not. Then it plans the next step. This allow to complete the task without the human help.
<br />
Thus, the smallest logical unit is not the agentic turn, but a single step. This is more similar to how a human thinks.

### Algorithm

1. **Start with the big picture**. The agent works from the top down. For example, it designs the code architecture first and improves it until the design is good. Then it plans the next, more detailed steps.
2. **Take small steps, one at a time**. Small steps allow the agent check if he moves toward the goal.
3. **Show proof**. Each step gives evidence that you can check later, such as a test run, a screenshot for pixel comparison or a source citation.
4. **Write down the lessons**. Agent record each step that was wrong, was replaced or took too much time.
5. **Reuse what worked**. At the start of the next task, the agent reads its earlier trails and uses the lessons from them.

### For example

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
