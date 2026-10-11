//! The help of `celis`: the page, and the page of each command.

/// The page: the reference for a person.
pub const PAGE: &str = r#"celis keeps how you do your tasks. A trail is one task you give, the steps
agents take toward it, and the decisions made on the way. Use it through the
/celis:trail skill.

Commands
  read     this project's active trails, open steps and decisions
  decide   a decision, in its scope
  tree     a trail and its steps, with their grades
  step     the steps of a trail: add, start, done, steer, drop, edit
  watch    a trail's tree, printed again on each change
  home     the folder of the store, to open it

$CELIS_HOME holds the key and the store. $CELIS_TRAIL names the trail new steps
go under.

See `celis <command> --help`."#;

/// The page the words of a command line name, before any `--`; none when they run a command.
pub fn named(args: &[String]) -> Option<String> {
    let before: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .take_while(|a| *a != "--")
        .collect();
    let words: Vec<&str> = before
        .iter()
        .copied()
        .filter(|a| !a.starts_with("--"))
        .collect();
    if words.is_empty() || words[0] == "help" {
        return Some(PAGE.to_string());
    }
    before
        .contains(&"--help")
        .then(|| Verb::of(&words).map_or(PAGE.to_string(), Verb::page))
}

/// The help of one command: its usage, what it does, its arguments and its examples.
pub struct Verb {
    pub command: &'static str,
    pub usage: &'static [&'static str],
    pub summary: &'static str,
    pub args: &'static [(&'static str, &'static str)],
    pub examples: &'static [&'static str],
}

impl Verb {
    /// The command the words begin with, the longest when two match.
    pub fn of(words: &[&str]) -> Option<&'static Verb> {
        VERBS
            .iter()
            .filter(|v| {
                let names: Vec<&str> = v.command.split(' ').collect();
                words.starts_with(&names)
            })
            .max_by_key(|v| v.command.len())
    }

    /// The command page: the usage, the summary, a line for each argument, and the examples.
    pub fn page(&self) -> String {
        let width = self.args.iter().map(|(n, _)| n.len()).max().unwrap_or(0);
        let mut lines: Vec<String> = self.usage.iter().flat_map(|u| wrap(u)).collect();
        lines.push(String::new());
        lines.extend(fill(self.summary));
        if !self.args.is_empty() {
            lines.push(String::new());
        }
        for (name, summary) in self.args {
            lines.push(format!("  {name:width$}    {summary}"));
        }
        lines.push(String::new());
        lines.push("Examples".to_string());
        lines.extend(self.examples.iter().map(|e| format!("  {e}")));
        lines.join("\n")
    }
}

/// The help of each command.
pub const VERBS: &[Verb] = &[
    Verb {
        command: "read",
        usage: &["celis read [--at <place>]... [--like <text>] [--full]"],
        summary: "Print this project's active trails and their rules, open steps and decisions; with --like, the past asks and trails like the text.",
        args: &[
            ("--at <place>", "a place whose decisions come first; zero or more"),
            ("--like <text>", "a new ask; prints the past work like it; optional"),
            ("--full", "print each past trail as tree --full does; optional"),
        ],
        examples: &[
            "celis read --at shelf/prices.py",
            "celis read --like \"Add an Invoices page from the frame\"",
        ],
    },
    Verb {
        command: "decide",
        usage: &["celis decide [--scope <scope>] [--replaces <id>] -- <text>"],
        summary: "Leave a decision, in its scope.",
        args: &[
            ("--scope <scope>", "at:<place>, trail:<id>, project or me; default: project"),
            ("--replaces <id>", "the live decision it takes the place of; optional"),
            ("<text>", "when it applies, what to do, and why"),
        ],
        examples: &[
            "celis decide --scope at:shelf/prices.py \\",
            "  -- \"When you change prices: keep them in cents.\"",
        ],
    },
    Verb {
        command: "tree",
        usage: &["celis tree [<id>] [--full] [--all]"],
        summary: "Print a trail with its state, its rules and its steps with their grades; with no id, the active trails and those from today.",
        args: &[
            ("<id>", "the step id, or a start of 4 or more characters; optional"),
            ("--full", "add the proof, grade and reason under each step; optional"),
            ("--all", "every trail of every project, by day, in place of an id; optional"),
        ],
        examples: &["celis tree 3fa2c1d0 --full", "celis tree --all"],
    },
    Verb {
        command: "watch",
        usage: &["celis watch [<id>] [--full]"],
        summary: "Print a trail's tree, and print it again each time it changes, until you stop it; with no id, every active trail of every project.",
        args: &[
            ("<id>", "the step id, or a start of 4 or more characters; optional"),
            ("--full", "add the proof, grade and reason under each step; optional"),
        ],
        examples: &["celis watch 3fa2c1d0"],
    },
    Verb {
        command: "home",
        usage: &["celis home"],
        summary: "Print the folder of the store, which holds every trail, step and decision as a file. On a Mac, open it with: open \"$(celis home)\"",
        args: &[],
        examples: &["celis home"],
    },
    Verb {
        command: "prompt",
        usage: &["celis prompt"],
        summary: "Read a hook's JSON on stdin, and record the person's prompt on the open trail of its session: a steer on the working step, else a prompt mark. A hook runs it on each prompt.",
        args: &[],
        examples: &["celis prompt < hook.json"],
    },
    Verb {
        command: "step add",
        usage: &["celis step add [--under <id>] [--fixes <id>] [--at <place>]... [--check <command>] [--new] -- <text>"],
        summary: "Leave a step. With no --under, it goes under $CELIS_TRAIL, else under the session's trail, else it starts a trail.",
        args: &[
            ("--under <id>", "the open step it is a part of; optional"),
            ("--fixes <id>", "a done step whose work it fixes; optional"),
            ("--at <place>", "file:line, a URL or any text; zero or more"),
            ("--check <command>", "the command that proves the work; optional"),
            ("--new", "a new trail, not a step of the session's trail; optional"),
            ("<text>", "a short title, then the work a stranger can do"),
        ],
        examples: &[
            "celis step add --under 3fa2c1d0 --check \"python3 -m unittest\" \\",
            "  -- \"A senior pays 80% of the price.\"",
        ],
    },
    Verb {
        command: "step start",
        usage: &["celis step start <id>"],
        summary: "Take up a step, and print the decisions at its places. It is working until it is done.",
        args: &[("<id>", "the step id, or a start of 4 or more characters")],
        examples: &["celis step start 4c1d22e0"],
    },
    Verb {
        command: "step done",
        usage: &["celis step done <id> [--pitfall <text>] -- <proof>"],
        summary: "Close a step with its proof, once the steps under it are done.",
        args: &[
            ("<id>", "the step id, or a start of 4 or more characters"),
            ("--pitfall", "what cost time, kept as a decision at the step's place"),
            ("<proof>", "the check that failed before and passes now"),
        ],
        examples: &[
            "celis step done 4c1d22e0 --pitfall \"The fixture needs make_fixtures --yes.\" \\",
            "  -- \"python3 -m unittest: test_senior failed before, passes now.\"",
        ],
    },
    Verb {
        command: "step steer",
        usage: &["celis step steer <id> -- <text>"],
        summary: "Record what the person asked for while a step was open.",
        args: &[
            ("<id>", "the step id, or a start of 4 or more characters"),
            ("<text>", "what the person asked, in one sentence"),
        ],
        examples: &["celis step steer 4c1d22e0 -- \"Use the shared Badge, not a new one.\""],
    },
    Verb {
        command: "step drop",
        usage: &["celis step drop <id> -- <reason>"],
        summary: "Close a step whose work is no longer wanted, and each open step under it.",
        args: &[
            ("<id>", "the step id, or a start of 4 or more characters"),
            ("<reason>", "why the work is no longer wanted"),
        ],
        examples: &["celis step drop 4c1d22e0 -- \"The person chose the shared table.\""],
    },
    Verb {
        command: "step edit",
        usage: &["celis step edit <id> [--at <place>]... [--check <command>] [-- <text>]"],
        summary: "Give a step a new title, places or check. A title alone keeps the detail.",
        args: &[
            ("<id>", "the step id, or a start of 4 or more characters"),
            ("--at <place>", "the places in place of the step's; zero or more"),
            ("--check <command>", "the check in place of the step's; optional"),
            ("<text>", "a new title, or a title and the detail; optional"),
        ],
        examples: &[
            "celis step edit 3fa2c1d0 -- \"Orange badges and buttons\"",
            "celis step edit 4c1d22e0 --check \"cargo test -p shop\"",
        ],
    },
];

/// The usage in lines of at most 80 characters, broken before a flag; a later line indents.
fn wrap(usage: &str) -> Vec<String> {
    let mut groups: Vec<String> = vec![];
    for word in usage.split(' ') {
        let flag = word.starts_with("[--") || word.starts_with("--");
        match groups.last_mut() {
            Some(group) if !flag => {
                group.push(' ');
                group.push_str(word);
            }
            _ => groups.push(word.to_string()),
        }
    }
    let mut lines: Vec<String> = vec![];
    for group in groups {
        match lines.last_mut() {
            Some(line) if line.len() + 1 + group.len() <= 80 => {
                line.push(' ');
                line.push_str(&group);
            }
            Some(_) => lines.push(format!("      {group}")),
            None => lines.push(group),
        }
    }
    lines
}

/// The text in lines of at most 80 characters, broken between words.
fn fill(text: &str) -> Vec<String> {
    let mut lines: Vec<String> = vec![];
    for word in text.split(' ') {
        match lines.last_mut() {
            Some(line) if line.len() + 1 + word.len() <= 80 => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
}

#[cfg(test)]
mod tests;
