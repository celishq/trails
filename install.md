# Install Celis

You are an AI agent, and your person asked you to install Celis. Celis keeps the corrections that your
person gives you as patterns, and gives them to every agent they use. It is open source (MIT):
https://github.com/celishq/trails. Do these steps in order.

1. Tell the person, in one or two sentences, that you will install the `celis` command and add its
   plugin to their agents. Before you run each command below, show it to the person.

2. Install the command:

   ```sh
   curl -fsSL https://celis.ai/install.sh | sh
   ```

   The script puts `celis` in `~/.local/bin` without `sudo`, checks the checksum of the download, and
   adds that folder to PATH in the shell's rc files. The person can read the script at the same URL
   first.

3. Run the setup. Use the full path, because your shell can start before PATH changes:

   ```sh
   ~/.local/bin/celis setup
   ```

4. Run each command that `celis setup` prints. When a command fails, show the person the command and its
   error, and give them https://github.com/celishq/trails#install, which shows how to add the plugin by
   hand.

5. Tell the person each thing that `celis setup` says only they can do, such as a restart of the agent.
   You cannot restart yourself, so stop there.

On Windows, do all of this in WSL. Celis keeps its data on this machine only. To see what Celis keeps,
sends and runs, read https://github.com/celishq/trails/blob/main/SECURITY.md.
