# Eval 54: install celis from install.md

The prompt is "Install Celis: follow install.md", with `install.md` pointed at a local release and a
stub for each agent's command. Each arm ran 5 times on claude-sonnet-5-5, on 31-10-2026.

| Grader        | install.sh with `mktemp -d` | install.sh with `mktemp -d "$TMPDIR/…"` |
| ------------- | --------------------------- | --------------------------------------- |
| ran-install   | 5/5                         | 5/5                                     |
| ran-setup     | 2/5                         | 5/5                                     |
| added-plugin  | 2/5                         | 5/5                                     |
| tells-restart | 2/5                         | 5/5                                     |
| Cost          | $0.54                       | $0.51                                   |

In the sandbox, `mktemp -d` on macOS chose a folder under `/var/folders`, which the sandbox does not let
the agent write. 3 runs stopped there; 2 worked around it. With the temp folder under `$TMPDIR`, every
run installed celis, ran `celis setup`, added the plugin and told the person to restart.
