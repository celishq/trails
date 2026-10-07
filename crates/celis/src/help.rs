//! The help pages of `celis`: the start page, the groups page, a group page, a command page and the
//! variables, as text or as JSON command objects.

mod group;
mod row;
mod topic;
mod variable;

pub use group::{Group, GROUPS};
use row::ROWS;
use serde_json::{json, Value};
use topic::Topic;
use variable::{Variable, VARIABLES};

/// The page a command line names, or none when it runs a command.
pub fn page(args: &[String]) -> Option<String> {
    let words: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|a| !a.starts_with("--"))
        .collect();
    let flag = args.iter().any(|a| a == "--help");
    let json = args.iter().any(|a| a == "--json");
    if args.iter().any(|a| a == "--version") {
        return Some(version(json));
    }
    let group = match words.as_slice() {
        ["help", word] | [word, ..] => Group::named(word),
        [] => None,
    };
    match (words.as_slice(), flag, group) {
        ([], false, _) => Some(shown(json, START, every())),
        (["help"], _, _) | ([], true, _) => Some(groups(json)),
        (["help", "env"], _, _) => Some(env(json)),
        (["cairn", "help", ..], _, Some(group)) => Some(group.page(json)),
        (["help", _] | [_], _, Some(group)) => Some(group.page(json)),
        (["help", _], _, None) => Some(groups(json)),
        ([_, command, ..], true, Some(group)) => Some(match group.topic(command) {
            Some(topic) if json => topic.json().to_string(),
            Some(topic) => topic.page(),
            None => group.page(json),
        }),
        (_, true, _) => Some(groups(json)),
        _ => None,
    }
}

/// The page as text, or its objects one a line under `--json`.
fn shown(json: bool, text: &str, objects: Vec<Value>) -> String {
    if json {
        let lines: Vec<String> = objects.iter().map(Value::to_string).collect();
        lines.join("\n")
    } else {
        text.to_string()
    }
}

/// The page `celis` with no words prints.
#[cfg(feature = "vault")]
const START: &str = "\
celis keeps a folder in sync across your computers, and keeps every version
of every file.

Start
  celis auth login              make an account, once on each computer
  celis vault create            make this folder a vault
  celis sync run                keep every vault in sync until you stop it

Agents
  celis cairn                   notes agents leave for each other

Run `celis help` for every group, or `celis help <group>` for one.";

/// The page `celis` with no words prints.
#[cfg(not(feature = "vault"))]
const START: &str = "\
celis links your coding agents. They leave notes for each other in a
folder, and your account owns each note they sign.

Start
  celis auth login              make an account, once on each computer
  celis cairn                   notes agents leave for each other

Run `celis help` for every group, or `celis help <group>` for one.";

/// The groups page: a line for each group and one for the variables.
fn groups(json: bool) -> String {
    let mut lines: Vec<String> = GROUPS
        .iter()
        .map(|g| format!("{:9}{}", g.name, g.summary))
        .collect();
    lines.push(format!("{:9}the variables celis reads", "env"));
    lines.push(String::new());
    lines.push("Run `celis help <group>` for its commands.".into());
    shown(json, &lines.join("\n"), every())
}

fn env(json: bool) -> String {
    let mut lines = vec![];
    for variable in VARIABLES {
        lines.push(format!("{:22}{}", variable.name, variable.summary));
        if let Some(default) = variable.default {
            lines.push(format!("{:22}default: {default}", ""));
        }
    }
    shown(json, &lines.join("\n"), variables())
}

fn every() -> Vec<Value> {
    let mut all: Vec<Value> = ROWS.iter().map(|r| r.topic.json()).collect();
    all.extend(
        celis_cairns::help::VERBS
            .iter()
            .map(|v| Topic::from(v).json()),
    );
    all.extend(variables());
    all
}

fn variables() -> Vec<Value> {
    VARIABLES.iter().map(Variable::json).collect()
}

fn version(json: bool) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let commit = option_env!("CELIS_COMMIT").map(|c| &c[..c.len().min(8)]);
    match (json, commit) {
        (true, _) => json!({"version": version, "commit": commit}).to_string(),
        (false, Some(commit)) => format!("celis {version} ({commit})"),
        (false, None) => format!("celis {version}"),
    }
}
