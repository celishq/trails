//! The help of `celis`: the page, and the page of each command.

use crate::failure::Failure;
use crate::terminal::canvas::{line, Styled, BOLD, DIM};
use ratatui::text::Span;

mod page;

/// Whether a command line gives its words to its command as they are: `grep` but `grep --help`.
pub fn raw(args: &[String]) -> bool {
    args.first().is_some_and(|a| a == "grep") && !args.iter().any(|a| a == "--help")
}

/// `lines` as text, each run of one look in its escape code when `colour`.
pub fn text(lines: &[Styled], colour: bool) -> String {
    let lines: Vec<String> = lines.iter().map(|l| line(l, colour)).collect();
    lines.join("\n")
}

/// The page the words of a command line name, before any `--`; none when they run a command.
pub fn named(args: &[String]) -> Option<Vec<Styled>> {
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
fn page_of(words: &[&str]) -> Vec<Styled> {
    let group = GROUPS.iter().find(|(name, ..)| words == [*name]);
    match (Verb::of(words), group) {
        (Some(verb), _) => verb.lines(),
        (None, Some(group)) => group_page(group),
        (None, None) => page::page(),
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
            ("start", "Start a step and show the rules it must keep"),
            ("done", "Finish a step with proof that it works"),
            ("drop", "Drop a step and the open steps under it"),
            (
                "edit",
                "Change a step's title, where it works, or its check",
            ),
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
fn group_page((name, summary, commands): &Group) -> Vec<Styled> {
    let mut lines = vec![
        Styled::from(vec![
            Span::styled(format!("celis {name}"), BOLD),
            Span::styled(" <command>", DIM),
        ]),
        Styled::default(),
    ];
    lines.extend(fill(summary).into_iter().map(Styled::raw));
    lines.push(Styled::default());
    lines.push(Styled::styled("Commands", BOLD));
    for (command, phrase) in *commands {
        lines.push(Styled::from(vec![
            Span::raw("  "),
            Span::styled(format!("{command:6}"), BOLD),
            Span::raw(format!("   {phrase}")),
        ]));
    }
    lines.push(Styled::default());
    lines.push(Styled::styled(
        format!("To learn more about a command, run `celis {name} <command> --help`."),
        DIM,
    ));
    lines
}

/// `name` with its first word bold and the rest dim, padded to `width` characters.
fn label(name: &str, width: usize) -> Vec<Span<'static>> {
    let (first, rest) = name.split_once(' ').unwrap_or((name, ""));
    let rest = match rest {
        "" => String::new(),
        rest => format!(" {rest}"),
    };
    let pad = " ".repeat(width.saturating_sub(name.chars().count()));
    vec![
        Span::styled(first.to_string(), BOLD),
        Span::styled(format!("{rest}{pad}"), DIM),
    ]
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

    /// The command page in its looks: the usage, the summary, its arguments and flags under their
    /// headings, and the examples.
    pub fn lines(&self) -> Vec<Styled> {
        let width = self.args.iter().map(|(n, _)| n.len()).max().unwrap_or(0);
        let bold = format!("celis {}", self.command);
        let mut lines: Vec<Styled> = self
            .usage
            .iter()
            .flat_map(|u| wrap(u))
            .map(|u| match u.strip_prefix(&bold) {
                Some(rest) => Styled::from(vec![
                    Span::styled(bold.clone(), BOLD),
                    Span::styled(rest.to_string(), DIM),
                ]),
                None => Styled::styled(u, DIM),
            })
            .collect();
        lines.push(Styled::default());
        lines.extend(fill(self.summary).into_iter().map(Styled::raw));
        for (heading, flags) in [("Arguments", false), ("Flags", true)] {
            let args: Vec<_> = self
                .args
                .iter()
                .filter(|(name, _)| name.starts_with("--") == flags)
                .collect();
            if args.is_empty() {
                continue;
            }
            lines.push(Styled::default());
            lines.push(Styled::styled(heading, BOLD));
            for (name, summary) in args {
                let mut row = vec![Span::raw("  ")];
                row.extend(label(name, width + 4));
                row.push(Span::raw(*summary));
                lines.push(Styled::from(row));
            }
        }
        lines.push(Styled::default());
        lines.push(Styled::styled("Examples", BOLD));
        lines.extend(self.examples.iter().map(|e| Styled::raw(format!("  {e}"))));
        lines
    }
}

/// The help of each command.
pub const VERBS: &[Verb] = &[
    Verb {
        command: "grep",
        usage: &["celis grep <grep option>... <pattern>"],
        summary: "Search every trail of every project with the system grep, one line for each trail, step, proof, change, drop reason, and rule. Show what it finds under each trail's header, the trails with the most lines first, up to 8,000 bytes. Changes nothing.",
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
        summary: "Show each step that later work reused and how often, then each rule that steps followed, then the total. With an id, show what that step reused and what reused it. On a terminal, show the totals of all time, then the last 14 days against the 14 before: reuse rate, first-pass success, resolved steps and median step time, then Learning, Quality, Output and speed, and Resources, each opened with enter, updating as work lands. Changes nothing.",
        args: &[
            ("<id>", "A step id, or its first 4 or more characters"),
            ("--demo", "Show four weeks of sample trails"),
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
        summary: "Keep this project's trails in a folder, and keep the folder in sync with the store. Use .celis/trails/ in the repo to commit them, or any folder, like a synced drive. Rules with the me scope stay private.",
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
        usage: &["celis decide [--scope <scope>] [--replaces <id>] [--project <id>] -- <text>"],
        summary: "Record a rule that later work must keep. With no --scope, it holds for the whole project.",
        args: &[
            ("--scope <scope>", "Where it applies: at:<place>, trail:<id>, me, or project"),
            ("--replaces <id>", "The rule this one replaces"),
            ("--project <id>", "skill:<name>, or the id of a project in the store"),
            ("<text>", "When it applies, what to do, and why"),
        ],
        examples: &[
            "celis decide --scope at:shelf/prices.py \\",
            "  -- \"When you change prices: keep them in cents.\"",
        ],
    },
    Verb {
        command: "name",
        usage: &["celis name <trail id> <name>"],
        summary: "Give a trail a name, so work in any project can use it: step add --uses <name> keeps its rules. The first trail with a name keeps it.",
        args: &[
            ("<trail id>", "A trail id, or its first 4 or more characters"),
            ("<name>", "A lowercase letter, then lowercase letters, digits, and -"),
        ],
        examples: &["celis name 3fa2c1d0 review"],
    },
    Verb {
        command: "names",
        usage: &["celis names [--limit <n>]"],
        summary: "Show each named trail of every project, as its name, id, and title, most used first. Changes nothing.",
        args: &[("--limit <n>", "Show the first <n> only")],
        examples: &["celis names --limit 20"],
    },
    Verb {
        command: "tree",
        usage: &["celis tree [<id>] [--graph] [--full] [--all] [--demo]"],
        summary: "Show a trail with its state, rules, and steps. With no id, show the active trails and today's trails of every project.",
        args: &[
            ("<id>", "A step id, or its first 4 or more characters"),
            ("--graph", "Show today's trails as live graphs of cards"),
            ("--full", "Show each step's proof, grade, and reason"),
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
        usage: &["celis step add [--under <id>] [--replaces <id>] [--uses <id>]... [--at <place>]... [--check <command>] [--new] [--session <id>] [--project <id>] -- <text>"],
        summary: "Add a step. With no --under, it goes under the session's trail, or starts a new trail.",
        args: &[
            ("--under <id>", "The open step this step is part of"),
            ("--replaces <id>", "A step this one takes over, or a finished step it fixes"),
            ("--uses <id>", "A rule or a past step it follows, or a trail's name"),
            ("--at <place>", "Where the work happens: a file:line, a URL, or any text"),
            ("--check <command>", "A command that proves the work"),
            ("--new", "Start a new trail, not a step of the session's trail"),
            ("--session <id>", "The session, when the hook names it"),
            ("--project <id>", "skill:<name>, or the id of a project in the store"),
            ("<text>", "What the person wants done, then its expect: line"),
        ],
        examples: &[
            "celis step add --under 3fa2c1d0 --check \"python3 -m unittest\" \\",
            "  -- \"Charge a senior 80% of the price\"",
        ],
    },
    Verb {
        command: "step start",
        usage: &["celis step start <id>"],
        summary: "Start a step and show the rules it must keep: those at its places, its trail's, and the ones its ask uses. The step stays in progress until it's done.",
        args: &[("<id>", "A step id, or its first 4 or more characters")],
        examples: &["celis step start 4c1d22e0"],
    },
    Verb {
        command: "step done",
        usage: &["celis step done <id> [--changed <text>] [--as-planned] [--pitfall <text>] -- <proof>"],
        summary: "Finish a step with its proof and what changed. Finish the steps under it first.",
        args: &[
            ("<id>", "A step id, or its first 4 or more characters"),
            ("--changed", "How the result differs from its expect line"),
            ("--as-planned", "The result matches its expect line; else use --changed"),
            ("--pitfall", "A rule from what went wrong, kept at the step's places"),
            ("<proof>", "The check that failed before and passes now"),
        ],
        examples: &[
            "celis step done 4c1d22e0 --as-planned --pitfall \\",
            "  \"When data is missing: run make_fixtures, because git has none.\" \\",
            "  -- \"python3 -m unittest: test_senior failed before, passes now.\"",
        ],
    },
    Verb {
        command: "step drop",
        usage: &["celis step drop <id> -- <reason>"],
        summary: "Drop a step, a done trail or a rule you no longer need, and each open step under it.",
        args: &[
            ("<id>", "A step, trail or rule id, or its first 4 or more characters"),
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
            ("--at <place>", "Where the work happens, in place of the old places"),
            ("--check <command>", "A command that proves the work, in place of the old one"),
            ("<text>", "A new title, or a title and details"),
        ],
        examples: &[
            "celis step edit 3fa2c1d0 -- \"Make the badges and buttons orange\"",
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
