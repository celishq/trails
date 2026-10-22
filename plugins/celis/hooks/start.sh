#!/bin/sh
# At the start of a Claude Code session: prints what celis gives the agent, how to install celis when it is
# missing, or how to update it when it is older than the plugin.
command -v celis >/dev/null || { echo 'The celis plugin needs the celis command, and none is on PATH. Tell the person to install it from https://celis.ai.'; exit 0; }
celis hook start claude "$CLAUDE_PLUGIN_ROOT" || echo 'The installed celis is older than the celis plugin. Tell the person to run: celis update'
