//! The help of `celis`: the page, and the page of each command.

/// The page: the reference for a person.
pub const PAGE: &str = r#"celis keeps how you do your tasks. A trail is one task you give, the steps
agents take toward it, and the decisions made on the way. Your agents use it
through the celis plugin.

Commands
  decide   a decision, in its scope
  tree     a trail and its steps, with their grades
  past     this project's active trails and decisions, every project's past work
  report   trails as Markdown, for a pull request
  issue    what went wrong, as Markdown for the celis team
  step     the steps of a trail: add, start, done, drop, edit
  auth     the account's phrase: login, status, phrase
  update   this binary, to the latest release or a version

$CELIS_HOME holds the phrase and the store.

See `celis <command> --help`."#;

/// The page the words of a command line name, before any `--`; none when they run a command.
pub fn named(args: &[String]) -> Option<String> {
    let words = words(args);
    if words.is_empty() || words[0] == "help" {
        return Some(PAGE.to_string());
    }
    if !args
        .iter()
        .take_while(|a| *a != "--")
        .any(|a| a == "--help")
    {
        return None;
    }
    let group = GROUPS.iter().find(|(name, _)| words == [*name]);
    Some(match (Verb::of(&words), group) {
        (Some(verb), _) => verb.page(),
        (None, Some((name, summary))) => group_page(name, summary),
        (None, None) => PAGE.to_string(),
    })
}

/// The help command for the words of a command line: `celis <command> --help`, else `celis help`.
pub fn see(args: &[String]) -> String {
    match Verb::of(&words(args)) {
        Some(verb) => format!("celis {} --help", verb.command),
        None => "celis help".to_string(),
    }
}

/// The words of a command line before any `--`, without its flags.
fn words(args: &[String]) -> Vec<&str> {
    args.iter()
        .map(String::as_str)
        .take_while(|a| *a != "--")
        .filter(|a| !a.starts_with("--"))
        .collect()
}

/// The commands whose name has two words, by their first word, with what the group holds.
pub const GROUPS: &[(&str, &str)] = &[
    (
        "step",
        "The steps of a trail: add one, then start, close, drop or edit it.",
    ),
    (
        "auth",
        "The recovery phrase of your account, kept in $CELIS_HOME/phrase.",
    ),
];

/// The page of the group `name`: its usage, `summary`, and each of its commands with the first sentence
/// of what it does, wrapped under its column.
fn group_page(name: &str, summary: &str) -> String {
    let verbs: Vec<&Verb> = VERBS
        .iter()
        .filter(|v| v.command.split(' ').next() == Some(name) && v.command.contains(' '))
        .collect();
    let width = verbs
        .iter()
        .map(|v| v.command.len() - name.len() - 1)
        .max()
        .unwrap_or(0);
    let mut lines = vec![format!("celis {name} <command>"), String::new()];
    lines.extend(fill(summary));
    lines.push(String::new());
    lines.push("Commands".to_string());
    for verb in verbs {
        let command = &verb.command[name.len() + 1..];
        let first = verb
            .summary
            .split(". ")
            .next()
            .unwrap_or("")
            .trim_end_matches('.');
        let indent = 2 + width + 4;
        let mut wrapped: Vec<String> = vec![];
        for word in first.split(' ') {
            match wrapped.last_mut() {
                Some(line) if indent + line.len() + 1 + word.len() <= 80 => {
                    line.push(' ');
                    line.push_str(word);
                }
                _ => wrapped.push(word.to_string()),
            }
        }
        for (n, part) in wrapped.iter().enumerate() {
            let head = if n == 0 { command } else { "" };
            lines.push(format!("  {head:width$}    {part}"));
        }
    }
    lines.push(String::new());
    lines.push(format!("See `celis {name} <command> --help`."));
    lines.join("\n")
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
        command: "past",
        usage: &["celis past [--at <place>]..."],
        summary: "Print this project's active trails with their rules and its decisions, then the done asks and reached trails of every project, newest first, one line each with its project, place, grade and day.",
        args: &[("--at <place>", "a place whose decisions come first; zero or more")],
        examples: &["celis past", "celis past --at shelf/prices.py"],
    },
    Verb {
        command: "report",
        usage: &["celis report <id>..."],
        summary: "Print the trail of each step the ids name as Markdown, for a pull request: its rules, and its steps with their places, checks, proofs and reasons. Writes no record.",
        args: &[("<id>", "a step id, or a start of 4 or more characters; one or more")],
        examples: &["celis report 3fa2c1d0", "celis report 3fa2c1d0 9e717e6f"],
    },
    Verb {
        command: "issue",
        usage: &["celis issue [<id>]... [-- <text>]"],
        summary: "Print what went wrong as Markdown for the celis team: the text, the version, system, project and store, the last lines of the log, and the trail of each step the ids name. Writes no record.",
        args: &[
            ("<id>", "a step id whose trail it adds; zero or more"),
            ("<text>", "what went wrong; optional"),
        ],
        examples: &[
            "celis issue -- \"celis prompt fails in each Cursor session\"",
            "celis issue 3fa2c1d0 -- \"The tree shows the step as open after done.\"",
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
        summary: "Print a trail with its state, its rules and its steps with their grades; with no id, the active trails and those from today, of every project.",
        args: &[
            ("<id>", "the step id, or a start of 4 or more characters; optional"),
            ("--full", "add the proof, grade and reason under each step; optional"),
            ("--all", "every trail of every project, by day, in place of an id; optional"),
        ],
        examples: &["celis tree 3fa2c1d0 --full", "celis tree --all"],
    },
    Verb {
        command: "prompt",
        usage: &["celis prompt"],
        summary: "Read a hook's JSON on stdin, and record the person's prompt on the open trail of its session: a steer on the working step, else a prompt mark. A hook runs it on each prompt.",
        args: &[],
        examples: &["celis prompt < hook.json"],
    },
    Verb {
        command: "auth login",
        usage: &["celis auth login"],
        summary: "Read a recovery phrase on stdin, keep it in $CELIS_HOME/phrase, and print the npub of its account. Every command signs as that account from now on. On a terminal it asks for the phrase, and does not show it.",
        args: &[],
        examples: &["celis auth login", "celis auth login < phrase.txt"],
    },
    Verb {
        command: "auth status",
        usage: &["celis auth status"],
        summary: "Print the npub of the account, and the path of its phrase. It never prints the phrase.",
        args: &[],
        examples: &["celis auth status"],
    },
    Verb {
        command: "auth phrase",
        usage: &["celis auth phrase"],
        summary: "Print the kept recovery phrase, to give it to the Celis app or to another machine.",
        args: &[],
        examples: &["celis auth phrase"],
    },
    Verb {
        command: "update",
        usage: &["celis update [<version>] [--dry-run]"],
        summary: "Replace this celis with the build of a release from GitHub, checked against its SHA-256. With no version, it takes the latest, and does nothing when that is installed.",
        args: &[
            ("<version>", "the version to install, as 1.2.0 or v1.2.0; optional"),
            ("--dry-run", "print the installed and the new version, and change nothing"),
        ],
        examples: &["celis update", "celis update 1.2.0", "celis update --dry-run"],
    },
    Verb {
        command: "step add",
        usage: &["celis step add [--under <id>] [--replaces <id>] [--at <place>]... [--check <command>] [--new] [--session <id>] -- <text>"],
        summary: "Leave a step. With no --under, it goes under the session's trail, else it starts a trail.",
        args: &[
            ("--under <id>", "the open step it is a part of; optional"),
            ("--replaces <id>", "an open step whose plan it takes over; optional"),
            ("--at <place>", "file:line, a URL or any text; zero or more"),
            ("--check <command>", "the command that proves the work; optional"),
            ("--new", "a new trail, not a step of the session's trail; optional"),
            ("--session <id>", "the session, when the hook names it; optional"),
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
        usage: &["celis step done <id> [--changed <text>] [--as-planned] [--pitfall <text>] -- <proof>"],
        summary: "Close a step with its proof and what changed, once the steps under it are done.",
        args: &[
            ("<id>", "the step id, or a start of 4 or more characters"),
            ("--changed", "how the work differs from the step's title"),
            ("--as-planned", "the work did what the title says; give it or --changed"),
            ("--pitfall", "what cost time, kept as a decision at the step's place"),
            ("<proof>", "the check that failed before and passes now"),
        ],
        examples: &[
            "celis step done 4c1d22e0 --as-planned --pitfall \"Run make_fixtures --yes.\" \\",
            "  -- \"python3 -m unittest: test_senior failed before, passes now.\"",
        ],
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
