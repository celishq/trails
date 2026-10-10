//! CIP-11, "Help".

use std::process::{Command, Output};

const CLI: &str = include_str!("../../../spec/trails/11-cli.md");
const STEPS: &str = include_str!("../../../spec/trails/12-steps.md");

fn run(args: &[&str]) -> Output {
    let home = tempfile::tempdir().unwrap();
    Command::new(env!("CARGO_BIN_EXE_celis"))
        .args(args)
        .env("CELIS_HOME", home.path())
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap()
}

/// What a help page prints: exit 0, the page on stdout, nothing on stderr.
fn page(args: &[&str]) -> String {
    let out = run(args);
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    assert_eq!(out.status.code(), Some(0), "{args:?}: {err:?}");
    assert!(err.is_empty(), "{args:?}: stderr {err:?}");
    String::from_utf8(out.stdout).unwrap()
}

/// The fenced blocks under `## <heading>` of a spec.
fn blocks(doc: &str, heading: &str) -> Vec<String> {
    let start = doc.find(&format!("\n## {heading}\n")).expect("section") + 1;
    let rest = &doc[start + 1..];
    let end = rest.find("\n## ").unwrap_or(rest.len());
    let mut found = Vec::new();
    let mut open: Option<Vec<&str>> = None;
    for line in rest[..end].lines() {
        match (&mut open, line.starts_with("```")) {
            (None, true) => open = Some(Vec::new()),
            (Some(_), true) => found.push(open.take().unwrap().join("\n")),
            (Some(lines), false) => lines.push(line),
            (None, false) => {}
        }
    }
    found
}

/// The page as the spec shows it.
fn the_page() -> String {
    blocks(CLI, "Help")[1].clone()
}

fn squeeze(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Each command of the spec with its usage: `read`, `decide` and `tree` of CIP-11, and the step
/// commands of CIP-12.
fn usages() -> Vec<(String, String)> {
    let mut found = Vec::new();
    let lines = blocks(CLI, "Commands")[0].clone() + "\n" + &blocks(STEPS, "The commands")[0];
    for line in lines
        .lines()
        .filter(|l| l.starts_with("celis ") && !l.contains(" | "))
    {
        let usage = line.split("   ").next().unwrap().trim().to_string();
        let words: Vec<&str> = usage.split(' ').collect();
        let command = match words[1] {
            "step" => format!("step {}", words[2]),
            word => word.to_string(),
        };
        found.push((command, usage));
    }
    found
}

/// The arguments and flags a usage names, a flag's value not counted.
fn names(usage: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut before = "";
    for word in usage.split_whitespace() {
        let token = word.trim_matches(|c| c == '[' || c == ']' || c == '.');
        let flag = token.starts_with("--") && token.len() > 2;
        let positional = token.starts_with('<') && !(before.starts_with("--") && before != "--");
        if (flag || positional) && !found.iter().any(|n| n == token) {
            found.push(token.to_string());
        }
        before = token;
    }
    found
}

/// `celis`, `celis help` and `celis --help` print the page as the spec shows it, and exit 0
/// (`spec/trails/11-cli.md`, "Help").
#[test]
fn celis_and_celis_help_print_the_page() {
    for args in [&[][..], &["help"], &["--help"]] {
        assert_eq!(page(args).trim_end(), the_page().trim_end(), "{args:?}");
    }
}

/// `--help` anywhere in a command prints the command page its words name, and else the page
/// (`spec/trails/11-cli.md`, "Help").
#[test]
fn help_anywhere_prints_the_command_page_its_words_name() {
    let commands = usages();
    assert_eq!(commands.len(), 14, "{commands:?}");
    for (command, usage) in &commands {
        let mut args: Vec<&str> = command.split(' ').collect();
        args.extend(["4c1d22e0", "--help", "--json"]);
        let shown = page(&args);
        assert!(
            squeeze(&shown).starts_with(&squeeze(usage)),
            "{command}:\n{shown}"
        );
    }
    for args in [&["nope", "--help"][..], &["step", "--help"]] {
        assert_eq!(page(args).trim_end(), the_page().trim_end(), "{args:?}");
    }
}

/// A command page gives one line to each argument and flag its usage names, and at least one example
/// (`spec/trails/11-cli.md`, "Help").
#[test]
fn a_command_page_gives_a_line_to_each_argument_and_an_example() {
    for (command, usage) in usages() {
        let mut args: Vec<&str> = command.split(' ').collect();
        args.push("--help");
        let shown = page(&args);
        let body: Vec<&str> = shown.lines().map(str::trim).collect();
        for name in names(&usage) {
            let lines = body
                .iter()
                .skip_while(|l| !l.is_empty())
                .filter(|l| {
                    !l.starts_with("celis ")
                        && l.strip_prefix(name.as_str())
                            .is_some_and(|r| r.is_empty() || r.starts_with(' '))
                })
                .count();
            assert_eq!(lines, 1, "{command}: lines for {name}:\n{shown}");
        }
        let at = body
            .iter()
            .position(|l| *l == "Examples")
            .expect("Examples");
        assert!(
            body[at + 1..].iter().any(|l| l.starts_with("celis ")),
            "{command}: no example:\n{shown}"
        );
    }
}

/// No line of a help page is longer than 80 characters, and no page names a spec file
/// (`spec/trails/11-cli.md`, "Help").
#[test]
fn every_help_page_fits_80_columns_and_names_no_spec() {
    let mut pages = vec![page(&["help"])];
    for (command, _) in usages() {
        let mut args: Vec<&str> = command.split(' ').collect();
        args.push("--help");
        pages.push(page(&args));
    }
    for shown in pages {
        for line in shown.lines() {
            assert!(line.chars().count() <= 80, "{line:?}");
        }
        assert!(!shown.contains("spec/"), "{shown}");
    }
}

/// `celis --version` prints `celis <version>` and the first 8 characters of the commit the build came
/// from (`spec/trails/11-cli.md`, "Help").
#[test]
fn celis_version_names_the_commit_the_build_came_from() {
    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8(o.stdout).unwrap().trim().to_string());
    let Some(head) = head else { return };
    let version = env!("CARGO_PKG_VERSION");
    assert_eq!(
        page(&["--version"]).trim_end(),
        format!("celis {version} ({})", &head[..8])
    );
}
