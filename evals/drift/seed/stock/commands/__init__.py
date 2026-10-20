"""The commands of the CLI, by name."""

from stock.commands.add import cmd_add
from stock.commands.list_items import cmd_list
from stock.commands.remove import cmd_remove

COMMANDS = {"add": cmd_add, "list": cmd_list, "remove": cmd_remove}
