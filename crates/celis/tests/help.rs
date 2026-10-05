//! CIP-05, "Help".

use std::process::{Command, Output};

const VAULT: &str = include_str!("../../../spec/vault/05-cli.md");
const CAIRNS: &str = include_str!("../../../spec/cairns/11-cli.md");
#[cfg(feature = "vault")]
const GROUPS: [&str; 5] = ["auth", "vault", "file", "sync", "cairn"];

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

#[cfg(feature = "vault")]
/// The block of the Help section of `doc` whose first line starts with `first`.
fn help_block(doc: &str, first: &str) -> String {
    blocks(doc, "Help")
        .into_iter()
        .find(|b| b.starts_with(first))
        .unwrap_or_else(|| panic!("no block starting {first:?}"))
}

fn squeeze(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Asserts that `items` appear in `page`, in order, ignoring how whitespace falls.
fn assert_in_order(page: &str, items: &[String]) {
    let text = squeeze(page);
    let mut from = 0;
    for item in items {
        let item = squeeze(item);
        let at = text[from..]
            .find(&item)
            .unwrap_or_else(|| panic!("{item:?} missing or out of order in:\n{page}"));
        from += at + item.len();
    }
}

struct Entry {
    key: String,
    usage: Vec<String>,
}

struct Variable {
    name: String,
    text: String,
    default: Option<String>,
}

/// The command lines of a "Commands" block, a command with several lines kept as one.
fn commands(doc: &str) -> Vec<Entry> {
    let mut found: Vec<Entry> = Vec::new();
    let mut part = String::new();
    for line in blocks(doc, "Commands")[0].lines() {
        if line.starts_with('$') {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (usage, summary) = match line.find("  ") {
            Some(at) => (&line[..at], true),
            None => (line, false),
        };
        part = format!("{part} {usage}").trim().to_string();
        if summary {
            let words: Vec<&str> = part.split(' ').collect();
            let key = format!("{} {}", words[1], words[2]);
            match found.iter_mut().find(|c| c.key == key) {
                Some(c) => c.usage.push(part.clone()),
                None => found.push(Entry {
                    key,
                    usage: vec![part.clone()],
                }),
            }
            part.clear();
        }
    }
    found
}

#[cfg(feature = "vault")]
/// Every command of CIP-05 then CIP-11, those of `celis cairn` last.
fn every_command() -> Vec<Entry> {
    let mut all = commands(VAULT);
    all.extend(commands(CAIRNS));
    all
}

/// The `$` lines of a "Commands" block, a variable with a second line kept as one.
fn variables(doc: &str) -> Vec<Variable> {
    let mut found: Vec<Variable> = Vec::new();
    for line in blocks(doc, "Commands")[0].lines() {
        if line.starts_with('$') {
            let name = line.split(' ').next().unwrap().to_string();
            found.push(Variable {
                name,
                text: line.to_string(),
                default: None,
            });
        } else if let (Some(last), false) = (found.last_mut(), line.trim().is_empty()) {
            last.text = format!("{} {}", last.text, line.trim());
            last.default = line.trim().strip_prefix("default: ").map(str::to_string);
        }
    }
    found
}

#[cfg(feature = "vault")]
fn every_variable() -> Vec<Variable> {
    let mut all = variables(VAULT);
    all.extend(variables(CAIRNS));
    all
}

#[cfg(feature = "vault")]
/// The lines of "Commands" that start with `celis <group>`, in order, summary kept.
fn group_lines(group: &str) -> Vec<String> {
    let doc = if group == "cairn" { CAIRNS } else { VAULT };
    let mut lines = Vec::new();
    let mut keep = false;
    for line in blocks(doc, "Commands")[0].lines() {
        if line.starts_with("celis ") {
            keep = line.starts_with(&format!("celis {group} "));
        } else if line.trim().is_empty() {
            keep = false;
        }
        if keep {
            lines.push(line.to_string());
        }
    }
    lines
}

fn command_page(command: &Entry) -> String {
    let words: Vec<&str> = command.key.split(' ').collect();
    page(&[words[0], words[1], "--help"])
}

/// The variables of CIP-05 the default build holds.
#[cfg(not(feature = "vault"))]
const KEPT: [&str; 3] = ["$CELIS_HOME", "$CELIS_HOME/key", "$CELIS_RELAY"];

/// The fenced blocks of "The default build": the start page, the groups page, the example.
#[cfg(not(feature = "vault"))]
fn default_blocks() -> Vec<String> {
    blocks(VAULT, "The default build")
}

/// The variables of CIP-05 the default build does not hold.
#[cfg(not(feature = "vault"))]
fn dropped() -> Vec<String> {
    variables(VAULT)
        .into_iter()
        .map(|v| v.name)
        .filter(|n| !KEPT.contains(&n.as_str()))
        .collect()
}

/// In the default build `help --json` holds every `auth` and `cairn` command, the kept CIP-05 variables and
/// every CIP-11 variable, and `help env` lists the CIP-11 variables (`spec/vault/05-cli.md`, "The default
/// build").
#[cfg(not(feature = "vault"))]
#[test]
fn the_default_build_holds_every_auth_and_cairn_command_and_kept_variable() {
    let lines = objects(&["help", "--json"]);
    let shown_commands: Vec<&str> = lines.iter().filter_map(|o| o["command"].as_str()).collect();
    let shown_env: Vec<&str> = lines.iter().filter_map(|o| o["env"].as_str()).collect();
    let auth = commands(VAULT)
        .into_iter()
        .filter(|c| c.key.starts_with("auth "));
    for command in auth.chain(commands(CAIRNS)) {
        assert!(
            shown_commands.contains(&command.key.as_str()),
            "{}",
            command.key
        );
    }
    let cip11 = variables(CAIRNS);
    let wanted = KEPT.iter().map(|k| k.to_string());
    for name in wanted.chain(cip11.iter().map(|v| v.name.clone())) {
        assert!(shown_env.contains(&name.as_str()), "{name}: {shown_env:?}");
    }
    let env: Vec<String> = cip11.into_iter().map(|v| v.text).collect();
    assert_in_order(&page(&["help", "env"]), &env);
}

/// In the default build `celis` and `celis help` print the default start and groups pages, `celis help env`
/// holds the kept CIP-05 variables, and `--json` covers only `auth`, `cairn` and those variables
/// (`spec/vault/05-cli.md`, "The default build").
#[cfg(not(feature = "vault"))]
#[test]
fn the_default_build_holds_only_auth_and_cairn() {
    let pages = default_blocks();
    assert_eq!(page(&[]).trim_end(), pages[0].trim_end());
    assert_eq!(page(&["help"]).trim_end(), pages[1].trim_end());
    let kept: Vec<String> = variables(VAULT)
        .into_iter()
        .filter(|v| KEPT.contains(&v.name.as_str()))
        .map(|v| v.text)
        .collect();
    assert_eq!(kept.len(), KEPT.len());
    assert_in_order(&page(&["help", "env"]), &kept);
    for object in objects(&["help", "--json"]) {
        if let Some(command) = object["command"].as_str() {
            assert!(
                command.starts_with("auth ") || command.starts_with("cairn "),
                "{command}"
            );
        }
        if let Some(env) = object["env"].as_str() {
            assert!(!dropped().contains(&env.to_string()), "{env}");
        }
    }
}

/// In the default build `vault`, `file` and `sync` are unknown groups, and no help page names them or a
/// CIP-05 variable the build drops (`spec/vault/05-cli.md`, "The default build").
#[cfg(not(feature = "vault"))]
#[test]
fn the_default_build_names_no_group_or_variable_it_lacks() {
    for group in ["vault", "file", "sync"] {
        let out = run(&[group]);
        let err = String::from_utf8_lossy(&out.stderr).to_string();
        assert_eq!(out.status.code(), Some(2), "{group}: {err:?}");
        assert!(out.stdout.is_empty(), "{group}");
        assert!(err.starts_with("celis: no command"), "{group}: {err:?}");
        assert_eq!(page(&[group, "--help"]), page(&["help"]), "{group}");
    }
    let mut pages = vec![page(&[]), page(&["help"]), page(&["help", "env"])];
    pages.push(page(&["auth"]));
    pages.push(page(&["cairn"]));
    let auth = commands(VAULT)
        .into_iter()
        .filter(|c| c.key.starts_with("auth "));
    pages.extend(auth.map(|c| command_page(&c)));
    pages.extend(commands(CAIRNS).iter().map(command_page));
    for shown in pages {
        for group in ["vault", "file", "sync"] {
            assert!(
                !shown.contains(&format!("celis {group}")),
                "{group}:\n{shown}"
            );
        }
        for name in dropped() {
            assert!(!shown.contains(&name), "{name}:\n{shown}");
        }
    }
}

/// In the default build `celis vault list` prints the error of the example and exits 2
/// (`spec/vault/05-cli.md`, "The default build").
#[cfg(not(feature = "vault"))]
#[test]
fn the_default_build_refuses_celis_vault_list() {
    let example = default_blocks()[2].clone();
    let mut lines = example.lines();
    let args: Vec<&str> = lines.next().unwrap()["$ celis ".len()..]
        .split(' ')
        .collect();
    let want = lines.next().unwrap();
    let out = run(&args);
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    assert_eq!(err.trim_end(), want);
    assert_eq!(out.status.code(), Some(2), "{err:?}");
}

/// `celis` with no words prints the start page exactly and exits 0; `celis help` prints the groups page
/// exactly; `celis help env` prints the `$` lines of CIP-05 then those of CIP-11 (`spec/vault/05-cli.md`,
/// "Help").
#[cfg(feature = "vault")]
#[test]
fn celis_with_no_words_prints_the_start_page() {
    let start = help_block(VAULT, "celis keeps");
    assert_eq!(page(&[]).trim_end(), start.trim_end());
    let groups = help_block(VAULT, "auth     your");
    assert_eq!(page(&["help"]).trim_end(), groups.trim_end());
    let env: Vec<String> = every_variable().into_iter().map(|v| v.text).collect();
    assert_in_order(&page(&["help", "env"]), &env);
}

/// `celis help <group>` and `celis <group>` print the group page, `--help` anywhere prints the narrowest page
/// its words name, and every help page exits 0 on stdout (`spec/vault/05-cli.md`, "Help").
#[cfg(feature = "vault")]
#[test]
fn help_vault_and_vault_alone_print_the_group_page() {
    for group in GROUPS.into_iter().filter(|g| *g != "cairn") {
        let shown = page(&["help", group]);
        assert_in_order(&shown, &group_lines(group));
        assert_eq!(page(&[group]), shown, "{group}");
        assert_eq!(page(&[group, "--bogus", "--help"]), shown, "{group}");
    }
    let command = help_block(VAULT, "celis vault create [<name>]");
    assert_eq!(
        page(&["vault", "create", "--help"]).trim_end(),
        command.trim_end()
    );
    let groups = page(&["help"]);
    assert_eq!(page(&["nope", "--help"]), groups);
    assert_eq!(page(&["--help"]), groups);
}

#[cfg(feature = "vault")]
/// The arguments and flags a command's usage lines name, a flag's value not counted.
fn named(usage: &[String]) -> Vec<String> {
    let is_flag =
        |w: &str| w.starts_with("--") && w[2..].starts_with(|c: char| c.is_ascii_lowercase());
    let mut names = Vec::new();
    for line in usage {
        let words: Vec<&str> = line
            .split([' ', '[', ']'])
            .filter(|w| !w.is_empty())
            .collect();
        for (i, word) in words.iter().enumerate() {
            let value = i > 0 && is_flag(words[i - 1]);
            let name = word.trim_end_matches('.').to_string();
            if (is_flag(word) || word.starts_with('<') && !value) && !names.contains(&name) {
                names.push(name);
            }
        }
    }
    names
}

/// A command page holds each usage line, a line for each argument and flag, and an example; no help page line
/// passes 80 characters, and no page names `spec/` (`spec/vault/05-cli.md`, "Help").
#[cfg(feature = "vault")]
#[test]
fn a_command_page_holds_its_usage_arguments_and_examples() {
    let mut pages = vec![page(&[]), page(&["help"]), page(&["help", "env"])];
    pages.extend(GROUPS.iter().map(|g| page(&[g])));
    for command in every_command() {
        let shown = command_page(&command);
        assert_in_order(&shown, &command.usage);
        let body: Vec<&str> = shown.lines().map(str::trim).collect();
        for name in named(&command.usage) {
            let has = body.iter().any(|l| {
                !l.starts_with("celis ")
                    && l.strip_prefix(name.as_str())
                        .is_some_and(|r| r.is_empty() || r.starts_with(' '))
            });
            assert!(has, "{}: no line for {name}:\n{shown}", command.key);
        }
        let at = body
            .iter()
            .position(|l| *l == "Examples")
            .unwrap_or_else(|| panic!("{}: no Examples", command.key));
        assert!(
            body[at + 1..].iter().any(|l| !l.is_empty()),
            "{}: no example",
            command.key
        );
        pages.push(shown);
    }
    for shown in pages {
        for line in shown.lines() {
            assert!(line.chars().count() <= 80, "{line:?}");
        }
        assert!(!shown.contains("spec/"), "{shown}");
    }
}

fn objects(args: &[&str]) -> Vec<serde_json::Value> {
    page(args)
        .lines()
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("{args:?}: {l:?}: {e}")))
        .collect()
}

#[cfg(feature = "vault")]
fn assert_command_object(object: &serde_json::Value, command: &Entry) {
    assert_eq!(object["command"], command.key.as_str());
    let usage: Vec<String> = object["usage"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| squeeze(u.as_str().unwrap()))
        .collect();
    let want: Vec<String> = command.usage.iter().map(|u| squeeze(u)).collect();
    assert_eq!(usage, want, "{}", command.key);
    assert!(
        object["summary"].as_str().is_some_and(|s| !s.is_empty()),
        "{}",
        command.key
    );
    assert!(object["args"].is_array(), "{}", command.key);
    assert!(
        object["examples"].as_array().is_some_and(|e| !e.is_empty()),
        "{}",
        command.key
    );
}

/// Under `--json` the groups page prints one command object a line for every command, then one env object
/// for every variable; a command page prints its one object; `celis --version` prints `celis <version>`
/// (`spec/vault/05-cli.md`, "Help").
#[cfg(feature = "vault")]
#[test]
fn under_json_the_groups_page_prints_a_command_object_a_line() {
    let commands = every_command();
    let variables = every_variable();
    let lines = objects(&["help", "--json"]);
    assert_eq!(lines.len(), commands.len() + variables.len());
    for (object, command) in lines.iter().zip(&commands) {
        assert_command_object(object, command);
    }
    for (object, variable) in lines[commands.len()..].iter().zip(&variables) {
        assert_eq!(object["env"], variable.name.as_str());
        assert!(
            object["summary"].as_str().is_some_and(|s| !s.is_empty()),
            "{}",
            variable.name
        );
        assert_eq!(
            object.get("default").and_then(|d| d.as_str()),
            variable.default.as_deref(),
            "{}",
            variable.name
        );
    }
    let one = objects(&["vault", "create", "--help", "--json"]);
    assert_eq!(one.len(), 1);
    assert_command_object(
        &one[0],
        commands.iter().find(|c| c.key == "vault create").unwrap(),
    );

    let version = page(&["--version"]);
    assert!(
        version.starts_with(&format!("celis {}", env!("CARGO_PKG_VERSION"))),
        "{version:?}"
    );
    let json = objects(&["--version", "--json"]);
    assert_eq!(json.len(), 1);
    assert_eq!(json[0]["version"], env!("CARGO_PKG_VERSION"));
}

/// `celis --json` alone prints what `celis help --json` prints and exits 0 (`spec/vault/05-cli.md`, "Help").
#[cfg(feature = "vault")]
#[test]
fn celis_alone_under_json_prints_every_command_then_every_variable() {
    let lines = objects(&["--json"]);
    assert_eq!(lines.len(), every_command().len() + every_variable().len());
    assert_eq!(lines, objects(&["help", "--json"]));
}

/// `celis vault nope --help` prints the vault group page, a known group with an unknown command
/// (`spec/vault/05-cli.md`, "Help").
#[cfg(feature = "vault")]
#[test]
fn celis_vault_nope_help_prints_the_vault_group_page() {
    let group = page(&["vault"]);
    assert_in_order(&group, &group_lines("vault"));
    assert_eq!(page(&["vault", "nope", "--help"]), group);
    assert_ne!(group, page(&["help"]));
}

/// Under `--json`, `celis vault`, `celis help vault`, `celis help env` and `celis cairn` print only the
/// objects they cover; `celis cairn write --help --json` prints one object (`spec/vault/05-cli.md`, "Help").
#[cfg(feature = "vault")]
#[test]
fn under_json_a_group_page_prints_only_the_objects_it_covers() {
    let all = every_command();
    let vault: Vec<&Entry> = all.iter().filter(|c| c.key.starts_with("vault ")).collect();
    for args in [&["vault", "--json"][..], &["help", "vault", "--json"][..]] {
        let lines = objects(args);
        assert_eq!(lines.len(), vault.len(), "{args:?}");
        for (object, command) in lines.iter().zip(&vault) {
            assert_command_object(object, command);
        }
    }
    let env = objects(&["help", "env", "--json"]);
    let variables = every_variable();
    assert_eq!(env.len(), variables.len());
    for (object, variable) in env.iter().zip(&variables) {
        assert_eq!(object["env"], variable.name.as_str());
        assert!(object.get("command").is_none(), "{}", variable.name);
    }
    let cairn: Vec<&Entry> = all.iter().filter(|c| c.key.starts_with("cairn ")).collect();
    let lines = objects(&["cairn", "--json"]);
    assert_eq!(lines.len(), cairn.len());
    for (object, command) in lines.iter().zip(&cairn) {
        assert_command_object(object, command);
    }
    let one = objects(&["cairn", "write", "--help", "--json"]);
    assert_eq!(one.len(), 1);
    assert_command_object(
        &one[0],
        cairn.iter().find(|c| c.key == "cairn write").unwrap(),
    );
}

/// `celis --version` names the commit the build came from, the first 8 characters of `git rev-parse HEAD`
/// in the repo, and `commit` under `--json` (`spec/vault/05-cli.md`, "Help").
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
    let commit = &head[..8];
    let version = env!("CARGO_PKG_VERSION");
    assert_eq!(
        page(&["--version"]).trim_end(),
        format!("celis {version} ({commit})")
    );
    let json = objects(&["--version", "--json"]);
    assert_eq!(json.len(), 1);
    assert_eq!(json[0]["commit"], commit);
}

/// `celis cairn` words follow CIP-11 and its help pages follow "Help": `celis cairn list --help` is a command
/// page of a CIP-11 command (`spec/vault/05-cli.md`, "Commands").
#[test]
fn celis_cairn_words_follow_cip_11_and_its_help_pages_follow_help() {
    let cairn: Vec<Entry> = commands(CAIRNS);
    assert!(!cairn.is_empty());
    for command in &cairn {
        assert!(command.key.starts_with("cairn "), "{}", command.key);
        let shown = command_page(command);
        assert_in_order(&shown, &command.usage);
        assert!(shown.lines().any(|l| l.trim() == "Examples"), "{shown}");
    }
    let list = cairn.iter().find(|c| c.key == "cairn list").unwrap();
    assert!(command_page(list).starts_with("celis cairn list"));
}

/// The pages of `celis cairn stats` and `celis cairn log` name the file they read
/// (`spec/cairns/11-cli.md`, "Help").
#[test]
fn the_pages_of_cairn_stats_and_log_name_the_log_file() {
    for word in ["stats", "log"] {
        let shown = page(&["cairn", word, "--help"]);
        assert!(shown.contains(".celis/cairns/local/log"), "{shown}");
    }
}

/// The cairn page prints as the spec shows it, in every build
/// (`spec/cairns/11-cli.md`, "Help").
#[test]
fn the_cairn_page_matches_the_spec() {
    let cairn = blocks(CAIRNS, "Help")[0].clone();
    assert_eq!(page(&["cairn"]).trim_end(), cairn.trim_end());
}

/// `celis cairn help` prints the cairn page and exits 0
/// (`spec/cairns/11-cli.md`, "Help").
#[test]
fn celis_cairn_help_must_print_the_cairn_page_and_exit_0() {
    let cairn = blocks(CAIRNS, "Help")[0].clone();
    assert_eq!(page(&["cairn", "help"]).trim_end(), cairn.trim_end());
}
