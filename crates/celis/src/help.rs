//! The help of `celis`: the page, and the page of each command.

use celis_trails::Failure;

/// The page: the reference for a person.
pub const PAGE: &str = r#"celis remembers how your agents do your tasks. Each task you give becomes a
trail: the steps your agents take and the decisions they make along the way.
Your agents use celis through the celis plugin.

Record work (your agents run these)
  step     Add, start, finish, drop, or edit a step
  decide   Record a decision

Read
  tree     Show a trail and its steps
  grep     Search every trail with grep
  stats    Show what reused work saved
  report   Show trails as Markdown for a pull request

Set up
  link     Keep this project's trails in a folder, like the repo or a drive
  unlink   Stop keeping this project's trails in a folder
  auth     Manage your recovery phrase
  update   Update celis to the latest version, or to the one you choose
  issue    Write a problem report for the celis team

celis keeps your recovery phrase and records in $CELIS_HOME.

To learn more about a command, run `celis <command> --help`."#;

/// Whether a command line gives its words to its command as they are: `grep` but `grep --help`.
pub fn raw(args: &[String]) -> bool {
    args.first().is_some_and(|a| a == "grep") && !args.iter().any(|a| a == "--help")
}

/// The page the words of a command line name, before any `--`; none when they run a command.
pub fn named(args: &[String]) -> Option<String> {
    if raw(args) {
        return None;
    }
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

/// A group: its first word, what it holds, and each of its commands with what it does.
pub type Group = (
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str)],
);

/// The commands whose name has two words, by their first word: what the group holds, and each of its
/// commands with what it does.
pub const GROUPS: &[Group] = &[
    (
        "step",
        "Add steps to a trail, then start, finish, drop, or edit them.",
        &[
            ("add", "Add a step to a trail, or start a new trail"),
            ("start", "Start a step and show the decisions at its places"),
            ("done", "Finish a step with proof that it works"),
            (
                "drop",
                "Drop a step you no longer need, and the open steps under it",
            ),
            ("edit", "Change a step's title, places, or check"),
        ],
    ),
    (
        "auth",
        "Manage your recovery phrase. celis keeps it in $CELIS_HOME/phrase.",
        &[
            ("login", "Save a recovery phrase and use its account"),
            ("status", "Show your account and where its phrase is kept"),
            ("phrase", "Show your recovery phrase"),
        ],
    ),
];

/// The page of a group: its usage, what it holds, and each of its commands with what it does.
fn group_page((name, summary, commands): &Group) -> String {
    let mut lines = vec![format!("celis {name} <command>"), String::new()];
    lines.extend(fill(summary));
    lines.push(String::new());
    lines.push("Commands".to_string());
    for (command, phrase) in *commands {
        lines.push(format!("  {command:6}   {phrase}"));
    }
    lines.push(String::new());
    lines.push(format!(
        "To learn more about a command, run `celis {name} <command> --help`."
    ));
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
        command: "grep",
        usage: &["celis grep <grep option>... <pattern>"],
        summary: "Search every trail of every project with the system grep. Each trail, step, proof, change, drop reason, and rule is one line with its day, trail, project, kind, and id. Show the lines found under the header of their trail, the trails with the most lines first, at most 8,000 bytes, then the trails cut. Changes nothing.",
        args: &[
            ("<grep option>", "Any option of grep, such as -i, -E, -w, or -A 3"),
            ("<pattern>", "What to find, as grep reads it"),
        ],
        examples: &[
            "celis grep -i -E \"price|cents\"",
            "celis grep \" live {me} \"",
            "celis grep -A3 9b0e11aa",
        ],
    },
    Verb {
        command: "stats",
        usage: &["celis stats [<id>] [--demo]"],
        summary: "Show each step that later work reused and what that saved, then each decision that steps followed, then the total. With an id, show what that step reused and what reused it. On a terminal, show them as cards that update as work lands. Changes nothing.",
        args: &[
            ("<id>", "A step id, or its first 4 or more characters"),
            ("--demo", "Show sample trails of four weeks"),
        ],
        examples: &["celis stats", "celis stats 9b0e11aa", "celis stats --demo"],
    },
    Verb {
        command: "report",
        usage: &["celis report <id>... [--demo]"],
        summary: "Show the trail of each step as Markdown for a pull request: a table of the trail, a table of its steps with their proofs, then their places and checks, and its rules. On a terminal, show a card for each trail, and press m to copy the Markdown. Changes nothing.",
        args: &[
            ("<id>", "A step id, or its first 4 or more characters"),
            ("--demo", "Show a sample trail"),
        ],
        examples: &["celis report 3fa2c1d0", "celis report 3fa2c1d0 9e717e6f", "celis report --demo"],
    },
    Verb {
        command: "issue",
        usage: &["celis issue [<id>]... [-- <text>]"],
        summary: "Write a problem report for the celis team as Markdown. It includes your text, the version, system, project, and store, the end of the log, and the trail of each step you name. Changes nothing.",
        args: &[
            ("<id>", "A step whose trail to include"),
            ("<text>", "What went wrong"),
        ],
        examples: &[
            "celis issue -- \"celis prompt fails in each Cursor session\"",
            "celis issue 3fa2c1d0 -- \"The tree shows the step as open after done.\"",
        ],
    },
    Verb {
        command: "link",
        usage: &["celis link [<folder>]"],
        summary: "Keep this project's trails in a folder, and keep the folder in sync with the store. Use .celis/trails/ in the repo to commit them, or any folder, like a synced drive. Decisions with the me scope stay private.",
        args: &[("<folder>", "The folder (default: .celis/trails at the repo root)")],
        examples: &["celis link", "celis link ~/Drive/celis/shop"],
    },
    Verb {
        command: "unlink",
        usage: &["celis unlink [<folder>]"],
        summary: "Stop keeping this project's trails in a folder on this computer. The folder and its records aren't deleted.",
        args: &[("<folder>", "The folder (default: .celis/trails at the repo root)")],
        examples: &["celis unlink", "celis unlink ~/Drive/celis/shop"],
    },
    Verb {
        command: "decide",
        usage: &["celis decide [--scope <scope>] [--replaces <id>] -- <text>"],
        summary: "Record a decision that later work must follow, in its scope.",
        args: &[
            ("--scope <scope>", "at:<place>, trail:<id>, project, or me (default: project)"),
            ("--replaces <id>", "The decision this one replaces"),
            ("<text>", "When it applies, what to do, and why"),
        ],
        examples: &[
            "celis decide --scope at:shelf/prices.py \\",
            "  -- \"When you change prices: keep them in cents.\"",
        ],
    },
    Verb {
        command: "tree",
        usage: &["celis tree [<id>] [--full] [--all] [--demo]"],
        summary: "Show a trail with its state, rules, and steps. With no id, show the active trails and today's trails of every project.",
        args: &[
            ("<id>", "A step id, or its first 4 or more characters"),
            ("--full", "Also show each step's proof, grade, and reason"),
            ("--all", "Show every trail of every project, by day"),
            ("--demo", "Show sample trails in every state"),
        ],
        examples: &["celis tree 3fa2c1d0 --full", "celis tree --all"],
    },
    Verb {
        command: "prompt",
        usage: &["celis prompt"],
        summary: "Read a hook's JSON from stdin, and record the prompt on the open trail of its session. The plugin runs this on each prompt.",
        args: &[],
        examples: &["celis prompt < hook.json"],
    },
    Verb {
        command: "auth login",
        usage: &["celis auth login"],
        summary: "Save a recovery phrase from stdin in $CELIS_HOME/phrase, and show the npub of its account. celis signs every record with this account from now on. On a terminal, it asks for the phrase and hides what you type.",
        args: &[],
        examples: &["celis auth login", "celis auth login < phrase.txt"],
    },
    Verb {
        command: "auth status",
        usage: &["celis auth status"],
        summary: "Show your account's npub and where its phrase is kept. It never shows the phrase.",
        args: &[],
        examples: &["celis auth status"],
    },
    Verb {
        command: "auth phrase",
        usage: &["celis auth phrase"],
        summary: "Show your recovery phrase, so you can add it to the Celis app or another computer.",
        args: &[],
        examples: &["celis auth phrase"],
    },
    Verb {
        command: "update",
        usage: &["celis update [<version>]"],
        summary: "Replace celis with a release build from GitHub, using the install.sh of the latest release. With no version, install the latest one, or do nothing if it's already installed.",
        args: &[
            ("<version>", "The version to install, like 1.2.0 or v1.2.0"),
        ],
        examples: &["celis update", "celis update 1.2.0"],
    },
    Verb {
        command: "step add",
        usage: &["celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--at <place>]... [--check <command>] [--new] [--session <id>] -- <text>"],
        summary: "Add a step. With no --under, it goes under the session's trail, or starts a new trail.",
        args: &[
            ("--under <id>", "The open step this step is part of"),
            ("--replaces <id>", "A step this one takes over, or a finished step it fixes"),
            ("--uses <id>", "A decision this step follows, or a past step it reuses"),
            ("--at <place>", "A file:line, a URL, or any text"),
            ("--check <command>", "A command that proves the work"),
            ("--new", "Start a new trail, not a step of the session's trail"),
            ("--session <id>", "The session, when the hook names it"),
            ("<text>", "A short title, then details anyone can follow"),
        ],
        examples: &[
            "celis step add --under 3fa2c1d0 --check \"python3 -m unittest\" \\",
            "  -- \"A senior pays 80% of the price.\"",
        ],
    },
    Verb {
        command: "step start",
        usage: &["celis step start <id>"],
        summary: "Start a step and show the decisions at its places. The step stays in progress until it's done.",
        args: &[("<id>", "A step id, or its first 4 or more characters")],
        examples: &["celis step start 4c1d22e0"],
    },
    Verb {
        command: "step done",
        usage: &["celis step done <id> [--changed <text>] [--as-planned] [--pitfall <text>] -- <proof>"],
        summary: "Finish a step with its proof and what changed. Finish the steps under it first.",
        args: &[
            ("<id>", "A step id, or its first 4 or more characters"),
            ("--changed", "How the work differs from the step's title"),
            ("--as-planned", "The work matches the title. Use this or --changed"),
            ("--pitfall", "What cost time, saved as a decision at the step's place"),
            ("<proof>", "The check that failed before and passes now"),
        ],
        examples: &[
            "celis step done 4c1d22e0 --as-planned --pitfall \"Run make_fixtures --yes.\" \\",
            "  -- \"python3 -m unittest: test_senior failed before, passes now.\"",
        ],
    },
    Verb {
        command: "step drop",
        usage: &["celis step drop <id> -- <reason>"],
        summary: "Drop a step you no longer need, and each open step under it.",
        args: &[
            ("<id>", "A step id, or its first 4 or more characters"),
            ("<reason>", "Why you no longer need it"),
        ],
        examples: &["celis step drop 4c1d22e0 -- \"The person chose the shared table.\""],
    },
    Verb {
        command: "step edit",
        usage: &["celis step edit <id> [--at <place>]... [--check <command>] [-- <text>]"],
        summary: "Change a step's title, places, or check. A new title alone keeps the details.",
        args: &[
            ("<id>", "A step id, or its first 4 or more characters"),
            ("--at <place>", "Places that replace the step's places"),
            ("--check <command>", "A check that replaces the step's check"),
            ("<text>", "A new title, or a title and details"),
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
