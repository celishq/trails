//! The page of `celis help`: what celis is, and each command with its flags.

use super::{Verb, GROUPS};
use crate::terminal::canvas::{Styled, BOLD, DIM};
use ratatui::text::Span;
use std::path::Path;

/// The sections of the page: a heading, its note, and its commands with their arguments.
const SECTIONS: &[(&str, &str, &[&str])] = &[
    (
        "Read",
        "",
        &[
            "patterns",
            "tree [<id>]",
            "grep <pattern>",
            "stats [<id>]",
            "extract <id>",
        ],
    ),
    (
        "Record work",
        "your agents run these",
        &[
            "pattern",
            "step add",
            "step start <id>",
            "step done <id>",
            "step drop <id>",
            "step edit <id>",
            "step note <id>",
        ],
    ),
    (
        "Set up",
        "",
        &[
            "link [<folder>]",
            "auth login",
            "auth status",
            "auth phrase",
            "update [<version>]",
            "issue [<id>]...",
        ],
    ),
];

/// What each command of one word does, in a phrase.
const PHRASES: &[(&str, &str)] = &[
    ("tree", "Show your trails and their steps"),
    ("grep", "Search every trail with grep"),
    ("patterns", "Show the index of your patterns"),
    ("stats", "Show how the patterns are used, and how work went"),
    ("extract", "Show what an ask taught, to extract as patterns"),
    ("pattern", "Record a pattern that guides later work"),
    (
        "link",
        "Keep this project's trails in a folder to commit or share",
    ),
    (
        "update",
        "Update celis to the latest version, or to one you choose",
    ),
    ("issue", "Write a problem report for the celis team"),
];

/// The flags only the plugin's hooks pass, left off the page.
const HIDDEN: &[&str] = &["--session"];

/// The width of the column of command names with their arguments.
const NAMES: usize = 20;

/// The width of the column of flags with their values.
const FLAGS: usize = 18;

/// The page: what celis is, then each command with what it does and its flags, then the store folder
/// `home`, when known.
pub fn page(home: Option<&Path>) -> Vec<Styled> {
    let mut lines = vec![
        Styled::raw("celis remembers what your agents learn, and how they do your tasks."),
        Styled::default(),
    ];
    lines.push(Styled::raw(
        "Your agents record them through the celis plugin.",
    ));
    lines.push(Styled::default());
    lines.push(Styled::styled("Usage", BOLD));
    lines.push(Styled::from(vec![
        Span::raw("  "),
        Span::styled("celis", BOLD),
        Span::styled(" <command> [<args>] [flags]", DIM),
    ]));
    for (heading, note, commands) in SECTIONS {
        lines.push(Styled::default());
        let mut head = vec![Span::styled(heading.to_string(), BOLD)];
        if !note.is_empty() {
            head.push(Span::styled(format!(" · {note}"), DIM));
        }
        lines.push(Styled::from(head));
        for (n, command) in commands.iter().enumerate() {
            let row = row(command);
            let spaced = row.len() > 1 && n + 1 < commands.len();
            lines.extend(row);
            if spaced {
                lines.push(Styled::default());
            }
        }
    }
    lines.push(Styled::default());
    match home {
        Some(home) => {
            lines.push(Styled::styled(
                "celis keeps your recovery phrase and records in $CELIS_HOME:",
                DIM,
            ));
            lines.push(Styled::styled(format!("  {}", home.display()), DIM));
        }
        None => lines.push(Styled::styled(
            "celis keeps your recovery phrase and records in $CELIS_HOME.",
            DIM,
        )),
    }
    lines.push(Styled::default());
    lines.push(Styled::styled(
        "To learn more about a command, run `celis <command> --help`.",
        DIM,
    ));
    lines
}

/// The lines of `command`: its name and arguments and what it does, then a line for each flag a person
/// passes.
fn row(command: &str) -> Vec<Styled> {
    let words: Vec<&str> = command
        .split(' ')
        .take_while(|w| !w.starts_with(['<', '[']))
        .collect();
    let name = words.join(" ");
    let args = command[name.len()..].to_string();
    let pad = NAMES.saturating_sub(command.len());
    let mut lines = vec![Styled::from(vec![
        Span::raw("  "),
        Span::styled(name.clone(), BOLD),
        Span::styled(format!("{args}{}", " ".repeat(pad)), DIM),
        Span::raw(phrase(&name)),
    ])];
    let Some(verb) = Verb::of(&words) else {
        return lines;
    };
    let flags: Vec<_> = flags(verb)
        .into_iter()
        .filter(|(flag, ..)| !HIDDEN.contains(flag))
        .collect();
    for (n, (flag, value, said)) in flags.iter().enumerate() {
        let last = n + 1 == flags.len();
        let (join, under) = match last {
            true => ("  └ ", "    "),
            false => ("  ├ ", "  │ "),
        };
        let shown = flag.len() + value.map_or(0, |v| v.len() + 1);
        let mut spans = vec![Span::styled(join, DIM), Span::raw(*flag)];
        if let Some(value) = value {
            spans.push(Span::styled(format!(" {value}"), DIM));
        }
        spans.push(Span::raw(" ".repeat(FLAGS.saturating_sub(shown))));
        let text = fill_to(said, 80 - 4 - FLAGS);
        for (k, part) in text.into_iter().enumerate() {
            if k > 0 {
                lines.push(Styled::from(std::mem::take(&mut spans)));
                spans = vec![Span::styled(under, DIM), Span::raw(" ".repeat(FLAGS))];
            }
            spans.push(Span::styled(part, DIM));
        }
        lines.push(Styled::from(spans));
    }
    lines
}

/// `text` in lines of at most `width` characters, broken between words.
fn fill_to(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = vec![];
    for word in text.split(' ') {
        match lines.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
}

/// What `command` does, in a phrase.
fn phrase(command: &str) -> &'static str {
    let grouped = command.split_once(' ').and_then(|(group, name)| {
        let (_, _, commands) = GROUPS.iter().find(|(g, ..)| *g == group)?;
        commands.iter().find(|(n, _)| *n == name).map(|(_, p)| *p)
    });
    let single = PHRASES.iter().find(|(n, _)| *n == command).map(|(_, p)| *p);
    grouped.or(single).unwrap_or("")
}

/// The flags of `verb` as its usage names them: each one's name, its value, and what it does.
fn flags(verb: &Verb) -> Vec<(&'static str, Option<&'static str>, &'static str)> {
    let words: Vec<&'static str> = verb.usage.iter().flat_map(|u| u.split(' ')).collect();
    let mut flags = Vec::new();
    for (i, word) in words.iter().enumerate() {
        let flag = word
            .trim_start_matches('[')
            .trim_end_matches("...")
            .trim_end_matches(']');
        if !flag.starts_with("--") || flag == "--" {
            continue;
        }
        let value = words
            .get(i + 1)
            .map(|v| v.trim_end_matches("...").trim_end_matches(']'))
            .filter(|v| v.starts_with('<'));
        let said = verb
            .args
            .iter()
            .find(|(name, _)| name.split(' ').next() == Some(flag))
            .map_or("", |(_, said)| *said);
        flags.push((flag, value, said));
    }
    flags
}
