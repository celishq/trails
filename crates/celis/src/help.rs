//! The help of `celis`: the page, and the page of each command.

use celis_trails::Failure;

/// The page: the reference for a person.
pub const PAGE: &str = r#"celis keeps how you do your tasks. A trail is one task you give, the steps
agents take toward it, and the decisions made on the way. Your agents use it
through the celis plugin.

Record work (your agents run these)
  step     the steps of a trail: add, start, done, drop, edit
  decide   a decision, in its scope

Read
  tree     a trail and its steps, with their grades
  past     this project's active trails and decisions, every project's past work
  stats    what later work reused, and what it saved
  report   trails as Markdown, for a pull request

Set up
  link     this project's trails kept in a folder: the repo, or a synced drive
  unlink   this project's trails no longer kept in a folder
  auth     the account's phrase: login, status, phrase
  update   this binary, to the latest release or a version
  issue    what went wrong, as Markdown for the celis team

$CELIS_HOME holds the phrase and the store.

See `celis <command> --help`."#;

/// The page the words of a command line name, before any `--`; none when they run a command.
pub fn named(args: &[String]) -> Option<String> {
    let words = words(args);
    if words.first() == Some(&"help") {
        return Some(page_of(&words[1..]));
    }
    let asked = args
        .iter()
        .take_while(|a| *a != "--")
        .any(|a| a == "--help" || a == "-h");
    let group = words.len() == 1 && GROUPS.iter().any(|(name, ..)| words[0] == *name);
    (words.is_empty() || asked || group).then(|| page_of(&words))
}

/// The page of `words`: the command page they name, else the group page, else the page.
fn page_of(words: &[&str]) -> String {
    let group = GROUPS.iter().find(|(name, ..)| words == [*name]);
    match (Verb::of(words), group) {
        (Some(verb), _) => verb.page(),
        (None, Some(group)) => group_page(group),
        (None, None) => PAGE.to_string(),
    }
}

/// The help command for the words of a command line: `celis <command> --help`, else
/// `celis <group> --help`, else `celis help`.
pub fn see(args: &[String]) -> String {
    let words = words(args);
    let group = GROUPS.iter().find(|(name, ..)| words.first() == Some(name));
    match (Verb::of(&words), group) {
        (Some(verb), _) => format!("celis {} --help", verb.command),
        (None, Some((name, ..))) => format!("celis {name} --help"),
        (None, None) => "celis help".to_string(),
    }
}

/// The failure a `word` gives that names no command of `known`; it names the nearest, if one is within
/// two edits.
pub fn unknown<'a>(word: &str, known: impl IntoIterator<Item = &'a str>) -> Failure {
    let nearest = known
        .into_iter()
        .map(|k| (edits(word, k), k))
        .filter(|(distance, _)| *distance <= 2)
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, k)| k);
    let hint = nearest.map_or(String::new(), |n| format!("; did you mean \"{n}\"?"));
    Failure::usage(format!("no command \"{word}\"{hint}"))
}

/// The Levenshtein distance between two words.
fn edits(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, x) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, y) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (diagonal + usize::from(x != *y))
                .min(above + 1)
                .min(row[j] + 1);
            diagonal = above;
        }
    }
    row[b.len()]
}

/// The words of a command line before any `--`, without its flags and `-h`.
fn words(args: &[String]) -> Vec<&str> {
    args.iter()
        .map(String::as_str)
        .take_while(|a| *a != "--")
        .filter(|a| !a.starts_with("--") && *a != "-h")
        .collect()
}

/// A group: its first word, what it holds, and each of its commands with what it is.
pub type Group = (
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str)],
);

/// The commands whose name has two words, by their first word: what the group holds, and each of its
/// commands with what it is.
pub const GROUPS: &[Group] = &[
    (
        "step",
        "The steps of a trail: add one, then start, close, drop or edit it.",
        &[
            ("add", "a step, under a trail or as a new trail"),
            ("start", "a step taken up, with the decisions at its places"),
            ("done", "a step closed with its proof"),
            (
                "drop",
                "a step no longer wanted, and the open steps under it",
            ),
            ("edit", "a step's new title, places or check"),
        ],
    ),
    (
        "auth",
        "The recovery phrase of your account, kept in $CELIS_HOME/phrase.",
        &[
            (
                "login",
                "a recovery phrase to keep, and the npub of its account",
            ),
            ("status", "the account's npub, and the path of its phrase"),
            ("phrase", "the kept recovery phrase"),
        ],
    ),
];

/// The page of a group: its usage, what it holds, and each of its commands with what it is.
fn group_page((name, summary, commands): &Group) -> String {
    let mut lines = vec![format!("celis {name} <command>"), String::new()];
    lines.extend(fill(summary));
    lines.push(String::new());
    lines.push("Commands".to_string());
    for (command, phrase) in *commands {
        lines.push(format!("  {command:6}   {phrase}"));
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
        summary: "Print this project's active trails with their rules and its decisions, then the done asks and done trails of every project, newest first, one line each with its project, place, grade and day.",
        args: &[("--at <place>", "a place whose decisions come first; zero or more")],
        examples: &["celis past", "celis past --at shelf/prices.py"],
    },
    Verb {
        command: "stats",
        usage: &["celis stats [<id>]"],
        summary: "Print each step that later steps follow, with what they saved, then each decision steps used, then the total; with an id, the steps it follows and the steps that follow it. Writes no record.",
        args: &[("<id>", "the step id, or a start of 4 or more characters; optional")],
        examples: &["celis stats", "celis stats 9b0e11aa"],
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
        command: "link",
        usage: &["celis link [<folder>]"],
        summary: "Keep this project's trails in a folder, and sync it with the store both ways: .celis/trails/ in the repo, to commit them, or any folder, such as a synced drive. From now on, each command reads it and writes the project's records to it. Decisions with the me scope stay out.",
        args: &[("<folder>", "the folder; default: .celis/trails in the top folder")],
        examples: &["celis link", "celis link ~/Drive/celis/shop"],
    },
    Verb {
        command: "unlink",
        usage: &["celis unlink [<folder>]"],
        summary: "Stop keeping this project's trails in a folder on this machine. The folder and its records stay.",
        args: &[("<folder>", "the folder; default: .celis/trails in the top folder")],
        examples: &["celis unlink", "celis unlink ~/Drive/celis/shop"],
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
        usage: &["celis tree [<id>] [--full] [--all] [--demo]"],
        summary: "Print a trail with its state, its rules and its steps with their grades; with no id, the active trails and those from today, of every project.",
        args: &[
            ("<id>", "the step id, or a start of 4 or more characters; optional"),
            ("--full", "add the proof, grade and reason under each step; optional"),
            ("--all", "every trail of every project, by day, in place of an id; optional"),
            ("--demo", "made-up trails in every state, in place of the store; optional"),
        ],
        examples: &["celis tree 3fa2c1d0 --full", "celis tree --all"],
    },
    Verb {
        command: "prompt",
        usage: &["celis prompt"],
        summary: "Read a hook's JSON on stdin, and record the person's prompt on the open trail of its session. A hook runs it on each prompt.",
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
        usage: &["celis update [<version>]"],
        summary: "Replace this celis with the build of a release from GitHub, by the install.sh of the latest release. With no version, it takes the latest, and does nothing when that is installed.",
        args: &[
            ("<version>", "the version to install, as 1.2.0 or v1.2.0; optional"),
        ],
        examples: &["celis update", "celis update 1.2.0"],
    },
    Verb {
        command: "step add",
        usage: &["celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--at <place>]... [--check <command>] [--new] [--session <id>] -- <text>"],
        summary: "Leave a step. With no --under, it goes under the session's trail, else it starts a trail.",
        args: &[
            ("--under <id>", "the open step it is a part of; optional"),
            ("--replaces <id>", "a step it takes over, or a done step it fixes; optional"),
            ("--uses <id>", "a decision it keeps, or a past step it follows; any"),
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
