//! The help of `celis`: the page, and the page of each command.

use crate::failure::Failure;
use crate::terminal::canvas::{line, Styled, BOLD, DIM};
use ratatui::text::Span;
use std::path::Path;

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

/// The page the words of a command line name, before any `--`, with the store folder `home`; none
/// when they run a command.
pub fn named(args: &[String], home: Option<&Path>) -> Option<Vec<Styled>> {
    if raw(args) {
        return None;
    }
    let words = words(args);
    if words.first() == Some(&"help") {
        return Some(page_of(&words[1..], home));
    }
    let asked = args
        .iter()
        .take_while(|a| *a != "--")
        .any(|a| a == "--help" || a == "-h");
    let group = words.len() == 1 && GROUPS.iter().any(|(name, ..)| words[0] == *name);
    (words.is_empty() || asked || group).then(|| page_of(&words, home))
}

/// The page of `words`: the command page they name, else the group page, else the page.
fn page_of(words: &[&str], home: Option<&Path>) -> Vec<Styled> {
    let group = GROUPS.iter().find(|(name, ..)| words == [*name]);
    match (Verb::of(words), group) {
        (Some(verb), _) => verb.lines(),
        (None, Some(group)) => group_page(group),
        (None, None) => page::page(home),
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
        "Add steps to a trail, then start, finish, drop, edit, or note them.",
        &[
            ("add", "Add a new ask, or a step under one"),
            (
                "start",
                "Start a step and show the patterns and notes it keeps",
            ),
            ("done", "Finish a step with proof that it works"),
            ("drop", "Drop a step and the open steps under it"),
            (
                "edit",
                "Change a step's title, where it works, or its check",
            ),
            ("note", "Leave a note for the work of the step's trail"),
        ],
    ),
    (
        "cell",
        "Group your patterns into cells, and share them.",
        &[
            ("add", "Put your patterns into a cell of yours"),
            ("on", "Use a cell's patterns in this project"),
            ("off", "Stop using a cell's patterns in this project"),
            ("export", "Write a cell of yours as a folder to share"),
            ("import", "Read a cell from a folder or a URL, and keep it"),
            ("remove", "Remove a cell of another account from disk"),
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
    (
        "hook",
        "What the plugin's hooks run. Each reads the hook's JSON from stdin.",
        &[
            ("start", "Print what the agent reads at a session start"),
            ("ask", "Record a prompt and print the note the agent reads"),
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
        summary: "Search every trail of every project with the system grep, one line for each trail, step, proof, change, drop reason, and pattern. Show what it finds under each trail's header, the trails with the most lines first. Changes nothing.",
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
        summary: "Show each live pattern: its uses, the steps it misled, its replaces, its last use, its examples and why it needs tidy. Then show the total: the size of the index, and the patterns written, the patterns replaced and the fixes that used no pattern in the last 7 days. Then show the lifecycle of the patterns by day, week and month, and the value of the last 28 days: asks that used a pattern, corrections and repeat ones, first-pass success with and without a pattern, recalls, uses, reach and promotions. With an id, show that step and its grade, and for a done ask its total: its steps, the patterns it used, and the patterns written since it began. On a terminal, show the totals of all time, then the last 14 days against the 14 before: reuse rate, first-pass success, resolved steps and median step time, then Learning, Quality, Output and speed, and Resources, each opened with enter, updating as work lands. Changes nothing.",
        args: &[
            ("<id>", "A step id, or its first 4 or more characters"),
            ("--demo", "Show four weeks of sample trails"),
        ],
        examples: &["celis stats", "celis stats 9b0e11aa", "celis stats --demo"],
    },
    Verb {
        command: "extract",
        usage: &["celis extract <id>"],
        summary: "Show what an ask or a step taught, open or done, to extract as patterns: what to keep, each change from its plan, each note of its trail, each prompt since the ask began, and each pattern its steps used. Give it to the celis:extraction agent. Changes nothing.",
        args: &[
            ("<id>", "A step id, or its first 4 or more characters"),
            ("--json", "Print the brief as one JSON object"),
        ],
        examples: &["celis extract 3fa2c1d0", "celis extract 3fa2c1d0 --json"],
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
            "celis issue -- \"celis hook ask fails in each Cursor session\"",
            "celis issue 3fa2c1d0 -- \"The tree shows the step as open after done.\"",
        ],
    },
    Verb {
        command: "link",
        usage: &["celis link [<folder>] [--off]"],
        summary: "Keep this project's trails in a folder, and keep the folder in sync with the store. Use .celis/trails/ in the repo to commit them, or any folder, like a synced drive. Patterns with the me scope stay private. With --off, stop keeping them there; the folder and its records stay.",
        args: &[
            ("<folder>", "The folder (default: .celis/trails at the repo root)"),
            ("--off", "Stop keeping the trails in the folder"),
        ],
        examples: &["celis link", "celis link ~/Drive/celis/shop", "celis link --off"],
    },
    Verb {
        command: "pattern",
        usage: &["celis pattern [--scope <scope>] [--replaces <id>] [--path <glob>]... [--within <id>]... [--good <example>]... [--bad <example>]... [--project <id>] -- <text>"],
        summary: "Record a pattern that guides later work: when it applies, its forces, what to do, and why. With no --scope, it holds for the whole project. Its paths name the files it acts on, so celis step done gives it for them. Where its forces are present, the work resolves them in its own way.",
        args: &[
            ("--scope <scope>", "Where it applies: me, or project"),
            ("--replaces <id>", "The pattern this one replaces"),
            ("--path <glob>", "A file it acts on; * and ? within a folder, ** across"),
            ("--within <id>", "The larger pattern it completes"),
            ("--good <example>", "A step, a file, or a URL that shows it, such as a pick"),
            ("--bad <example>", "A step, a file, or a URL that breaks it"),
            ("--project <id>", "The id of a project in the store"),
            ("<text>", "Its lines: Name, When, Forces, Do and Because"),
        ],
        examples: &[
            "celis pattern --path shelf/prices.py \\",
            "  -- \"When you change prices: keep them in cents.\"",
            "celis pattern --path \"*.svg\" --good logo.svg --bad 3fa2c1d0 \\",
            "  -- \"When a logo is drawn: keep sharp corners.\"",
        ],
    },
    Verb {
        command: "patterns",
        usage: &["celis patterns [<id>] [<file>]... [--markdown] [--tidy] [--cell <name>]"],
        summary: "Show the index of the patterns of this project and of you, the oldest first: each one's id, scope and When, with its paths, each named file they match, and the larger patterns it completes. With an id, print that pattern whole, with its examples, and mark it recalled on the session's open step. With --markdown, print each whole as one page, with its examples, each file a link to its blob, for a review or a skill file. With --tidy, list the patterns that need tidy, with the reason, then the size of the index.",
        args: &[
            ("<id>", "A pattern id, or its first 4 or more characters"),
            ("<file>", "A file or folder; marks each pattern whose paths match it"),
            ("--markdown", "Print one Markdown page, each file a link to its blob"),
            ("--tidy", "List the patterns that need tidy, and why"),
            ("--cell <name>", "Show only the patterns of one cell, on or off"),
        ],
        examples: &[
            "celis patterns",
            "celis patterns shelf/prices.py shelf/",
            "celis patterns 9b0e11aa",
            "celis patterns --markdown > patterns.md",
            "celis patterns --tidy",
            "celis patterns --cell eating-east",
        ],
    },
    Verb {
        command: "cells",
        usage: &["celis cells"],
        summary: "Show each cell: its name, the start of its account's npub, how many patterns it holds, and whether it is on in this project. Changes nothing.",
        args: &[],
        examples: &["celis cells"],
    },
    Verb {
        command: "tree",
        usage: &["celis tree [<id>]... [--graph] [--full] [--all] [--demo]"],
        summary: "Show a trail with its state, notes, and steps. With no id, show the active trails and today's trails of every project.",
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
        command: "cell add",
        usage: &["celis cell add <name> <id>..."],
        summary: "Put your patterns into a cell of yours, and make the cell when it is new. A new cell is off. A name that a cell of another account holds is refused.",
        args: &[
            ("<name>", "The cell: a-z, 0-9 and -, up to 64 characters"),
            ("<id>", "A pattern of yours, or its first 4 or more characters"),
        ],
        examples: &["celis cell add eating-east 9b0e11aa 4c2d17e0"],
    },
    Verb {
        command: "cell on",
        usage: &["celis cell on <name> [--scope me]"],
        summary: "Use a cell's patterns in this project: they join the index, whatever their own scope and paths. With --scope me, use them in every project where no cell on or cell off of that project says otherwise.",
        args: &[
            ("<name>", "The cell"),
            ("--scope", "me: in every project"),
        ],
        examples: &["celis cell on eating-east", "celis cell on eating-east --scope me"],
    },
    Verb {
        command: "cell off",
        usage: &["celis cell off <name> [--scope me]"],
        summary: "Stop using a cell's patterns in this project: they leave the index, and so do the patterns that replace them. With --scope me, stop using them in every project where no cell on or cell off of that project says otherwise.",
        args: &[
            ("<name>", "The cell"),
            ("--scope", "me: in every project"),
        ],
        examples: &["celis cell off eating-east", "celis cell off eating-east --scope me"],
    },
    Verb {
        command: "cell export",
        usage: &["celis cell export <name> [--to <folder>]"],
        summary: "Write a cell of yours as a folder to share: cell.md to read, with each pattern, its example files and the credits; cell.jsonl, the signed patterns; and blobs/, the example files. Trails, steps and notes stay out. Changes no record.",
        args: &[
            ("<name>", "A cell of yours"),
            ("--to <folder>", "The folder to write (default: ./<name>)"),
        ],
        examples: &["celis cell export eating-east", "celis cell export eating-east --to ~/cells/east"],
    },
    Verb {
        command: "cell import",
        usage: &["celis cell import <source> [--yes]"],
        summary: "Read a cell that another person exported, check that each of its records is signed and unchanged, and show its patterns. With --yes, keep it. A kept cell is off until you turn it on with celis cell on, and you change it only with your own patterns that replace its patterns.",
        args: &[
            ("<source>", "The cell's folder, or the URL of the folder or of its cell.jsonl"),
            ("--yes", "Keep the cell after you read its patterns"),
        ],
        examples: &[
            "celis cell import ~/Downloads/eating-east",
            "celis cell import https://example.com/cells/eating-east/ --yes",
        ],
    },
    Verb {
        command: "cell remove",
        usage: &["celis cell remove <name>"],
        summary: "Remove a cell that you imported from disk: its patterns, its memberships, and each example file that no other record names. Your own patterns stay, also those that replace its patterns. A cell of yours only turns off, because its records may be on relays.",
        args: &[("<name>", "A cell of another account")],
        examples: &["celis cell remove eating-east"],
    },
    Verb {
        command: "hook start",
        usage: &["celis hook start <agent> <plugin folder>"],
        summary: "Read a session start hook's JSON from stdin, and print what the agent reads: the card and a notice when celis and the plugin differ in version. The plugin runs this.",
        args: &[
            ("<agent>", "claude, codex, cursor, gemini or antigravity"),
            ("<plugin folder>", "The folder of the celis plugin"),
        ],
        examples: &["celis hook start claude ~/.claude/plugins/celis < hook.json"],
    },
    Verb {
        command: "hook ask",
        usage: &["celis hook ask <agent>"],
        summary: "Read a prompt hook's JSON from stdin, record the prompt on the open ask of its session, and print the note the agent reads. The plugin runs this on each prompt.",
        args: &[("<agent>", "claude, codex, cursor, gemini or antigravity")],
        examples: &["celis hook ask codex < hook.json"],
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
        usage: &["celis step add [--under <id>] [--new] [--replaces <id>] [--uses <id>]... [--misled <id>]... [--at <place>]... [--check <command>] [--session <id>] [--project <id>] -- <text>"],
        summary: "Add a step. With no --under, it is a new ask, and starts its trail.",
        args: &[
            ("--under <id>", "The open step or done ask this step is part of"),
            ("--new", "A new ask, beside the session's trail"),
            ("--replaces <id>", "A step this one takes over, or a finished step it fixes"),
            ("--uses <id>", "A pattern it follows"),
            ("--misled <id>", "A pattern that misled the work it fixes"),
            ("--at <place>", "Where the work happens: a file:line, a URL, or any text"),
            ("--check <command>", "A command that proves the work"),
            ("--session <id>", "The session, when the hook names it"),
            ("--project <id>", "The id of a project in the store"),
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
        summary: "Start a step and show what it keeps: the patterns its ask uses, and the notes of its trail. The step stays in progress until it's done.",
        args: &[("<id>", "A step id, or its first 4 or more characters")],
        examples: &["celis step start 4c1d22e0"],
    },
    Verb {
        command: "step done",
        usage: &["celis step done <id> [--changed <text>] [--as-planned] [--by person] [--misled <id>]... [--file <path>]... -- <proof>"],
        summary: "Finish a step with its proof and what changed. Finish the steps under it first.",
        args: &[
            ("<id>", "A step id, or its first 4 or more characters"),
            ("--changed", "How the result differs from its expect line"),
            ("--as-planned", "The result matches its expect line; else use --changed"),
            ("--by", "person: the person confirms the result"),
            ("--misled", "A pattern it followed that did not fit"),
            ("--file", "A file of the proof, kept as it is now"),
            ("<proof>", "The check that failed before and passes now"),
        ],
        examples: &[
            "celis step done 4c1d22e0 --as-planned \\",
            "  -- \"python3 -m unittest: test_senior failed before, passes now.\"",
        ],
    },
    Verb {
        command: "step drop",
        usage: &["celis step drop <id> -- <reason>"],
        summary: "Drop a step, a done trail or a pattern you no longer need, and each open step under it.",
        args: &[
            ("<id>", "A step, trail or pattern id, or its first 4 or more characters"),
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
    Verb {
        command: "step note",
        usage: &["celis step note <id> -- <text>"],
        summary: "Leave a note on a step: what holds for the work of its trail only. celis step start shows the notes of the step's trail.",
        args: &[
            ("<id>", "A step id, or its first 4 or more characters"),
            ("<text>", "What later steps of the trail must keep"),
        ],
        examples: &["celis step note 3fa2c1d0 -- \"Keep the old table until the release.\""],
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
