//! The plugin's hooks, run as each agent's manifest declares them.

use crate::support;

use celis_nostr::Note;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use support::{err, out, Repo};

/// One agent: the file that declares its hooks, the events it runs them on, the variable that names the
/// root, and whether the hook runs in the plugin root rather than the project. An empty `ask` is an agent
/// with no prompt hook, and an empty variable an agent that names no root.
struct Agent {
    name: &'static str,
    hooks: &'static str,
    start: &'static str,
    ask: &'static str,
    root: (&'static str, &'static str),
    in_root: bool,
}

const AGENTS: [Agent; 5] = [
    Agent {
        name: "claude",
        hooks: "plugins/celis/hooks/hooks.json",
        start: "SessionStart",
        ask: "UserPromptSubmit",
        root: ("CLAUDE_PLUGIN_ROOT", "plugins/celis"),
        in_root: false,
    },
    Agent {
        name: "codex",
        hooks: "plugins/celis/hooks/hooks-codex.json",
        start: "SessionStart",
        ask: "UserPromptSubmit",
        root: ("PLUGIN_ROOT", "plugins/celis"),
        in_root: false,
    },
    Agent {
        name: "cursor",
        hooks: "plugins/celis/hooks/hooks-cursor.json",
        start: "sessionStart",
        ask: "beforeSubmitPrompt",
        root: ("CURSOR_PLUGIN_ROOT", "plugins/celis"),
        in_root: true,
    },
    Agent {
        name: "gemini",
        hooks: "hooks/hooks.json",
        start: "SessionStart",
        ask: "BeforeAgent",
        root: ("extensionPath", ""),
        in_root: false,
    },
    Agent {
        name: "antigravity",
        hooks: "hooks.json",
        start: "PreInvocation",
        ask: "",
        root: ("", ""),
        in_root: true,
    },
];

fn checkout() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> serde_json::Value {
    let text = std::fs::read_to_string(checkout().join(rel)).unwrap();
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

impl Agent {
    /// The command the agent's hooks file declares for `event`.
    fn command(&self, event: &str) -> String {
        let file = read(self.hooks);
        let events = match self.name {
            "antigravity" => &file["celis"],
            _ => &file["hooks"],
        };
        let first = &events[event][0];
        let hook = if first["hooks"].is_array() {
            &first["hooks"][0]
        } else {
            first
        };
        let command = hook["command"].as_str().map(str::to_string);
        command.unwrap_or_else(|| panic!("{}: no {event} hook in {}", self.name, self.hooks))
    }

    /// The hook's JSON on stdin, as the agent writes it, for the session `session` in `project`.
    fn payload(&self, session: &str, project: &Path, prompt: &str) -> String {
        let project = project.to_str().unwrap();
        match self.name {
            "cursor" => serde_json::json!({"conversation_id": session, "session_id": "other",
                "workspace_roots": [project], "prompt": prompt}),
            "antigravity" => serde_json::json!({"conversationId": session,
                "workspacePaths": [project], "invocationNum": 0}),
            _ => serde_json::json!({"session_id": session, "cwd": project, "prompt": prompt}),
        }
        .to_string()
    }

    /// What the agent reads from the hook's stdout.
    fn context(&self, stdout: &str) -> String {
        let json = || serde_json::from_str::<serde_json::Value>(stdout).unwrap();
        match self.name {
            "cursor" => json()["additional_context"].as_str().unwrap().to_string(),
            "gemini" => json()["hookSpecificOutput"]["additionalContext"]
                .as_str()
                .unwrap()
                .to_string(),
            "antigravity" => json()["injectSteps"][0]["ephemeralMessage"]
                .as_str()
                .unwrap()
                .to_string(),
            _ => stdout.to_string(),
        }
    }

    /// Runs the hook of `event` with `stdin`, as the agent does, with `celis` on PATH.
    fn run(&self, repo: &Repo, event: &str, stdin: &str) -> String {
        let celis = Path::new(env!("CARGO_BIN_EXE_celis")).parent().unwrap();
        self.run_from(repo, event, stdin, celis)
    }

    /// Runs the hook of `event` with `stdin`, as the agent does, with `bin` first on PATH.
    fn run_from(&self, repo: &Repo, event: &str, stdin: &str, bin: &Path) -> String {
        let root = checkout().join(self.root.1);
        let variable = Some(self.root.0).filter(|v| !v.is_empty());
        let folder = if self.in_root {
            root.clone()
        } else {
            repo.path()
        };
        let mut child = Command::new("sh")
            .args(["-c", &self.command(event)])
            .current_dir(folder)
            .env_clear()
            .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
            .env("CELIS_HOME", repo.home())
            .envs(variable.map(|v| (v, &root)))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let written = std::io::Write::write_all(&mut child.stdin.take().unwrap(), stdin.as_bytes());
        if let Err(e) = written {
            assert_eq!(e.kind(), std::io::ErrorKind::BrokenPipe, "{e}");
        }
        let o = child.wait_with_output().unwrap();
        assert!(o.status.success(), "{} {event}: {}", self.name, err(&o));
        out(&o)
    }
}

/// The prompt marks on `step` in the store.
fn prompts(repo: &Repo, step: &str) -> Vec<String> {
    std::fs::read_dir(repo.store().join("shared"))
        .unwrap()
        .filter_map(|e| std::fs::read_to_string(e.unwrap().path()).ok())
        .filter_map(|t| serde_json::from_str::<Note>(&t).ok())
        .filter(|n| n.kind == 1985 && n.tag("l") == Some("prompt") && n.tag("e") == Some(step))
        .map(|n| n.content)
        .collect()
}

/// Each agent's session start gives the card; an agent other than Claude Code also gets its session, to
/// pass with `--session`, and the paths of `lesson.md` and `recall.md`. Antigravity has no session start, and gets both
/// before each call of the model.
#[test]
fn each_agents_session_start_gives_the_card_and_its_session() {
    let repo = Repo::new();
    for agent in AGENTS {
        let stdout = agent.run(&repo, agent.start, &agent.payload("s1", &repo.path(), ""));
        let context = agent.context(&stdout);
        assert!(
            context.starts_with("How to do an ask with the celis plugin:"),
            "{}: {context}",
            agent.name
        );
        let named = context.contains("celis session: s1. Pass --session s1");
        assert_eq!(named, agent.name != "claude", "{}: {context}", agent.name);
        let taught = context.contains("agents/lesson.md. With no subagents");
        assert_eq!(taught, agent.name != "claude", "{}: {context}", agent.name);
        let recalled = context.contains("agents/recall.md. With no subagents");
        assert_eq!(
            recalled,
            agent.name != "claude",
            "{}: {context}",
            agent.name
        );
    }
}

/// Each agent's session start gives, after the card, the index of the patterns in scope in the person's
/// project, under one heading line; none while the store holds no pattern (`spec/02-cli.md`, "Hooks").
#[test]
fn each_agents_session_start_gives_the_index_of_the_patterns() {
    let repo = Repo::new();
    let start = |agent: &Agent| {
        let stdout = agent.run(&repo, agent.start, &agent.payload("s1", &repo.path(), ""));
        agent.context(&stdout)
    };
    for agent in AGENTS {
        assert!(
            !start(&agent).contains("The patterns in scope here:"),
            "{}",
            agent.name
        );
    }
    let o = repo.run(
        &repo.path(),
        &["pattern", "--", "When you test: set A=1."],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    let id = out(&o).lines().next().unwrap().to_string();
    let index = format!(
        "The patterns in scope here:\n{} project When you test",
        &id[..8]
    );
    for agent in AGENTS {
        let context = start(&agent);
        assert!(context.contains(&index), "{}: {context}", agent.name);
    }
}

/// Claude Code's session start asks to offer `/celis:tidy` while the store holds a pattern and no step whose
/// title starts with "Tidy the celis trails" began in the last 7 days.
#[test]
fn claudes_session_start_asks_to_offer_a_tidy() {
    let repo = Repo::new();
    let claude = &AGENTS[0];
    let start = claude.payload("s1", &repo.path(), "");
    assert!(!claude
        .run(&repo, claude.start, &start)
        .contains("/celis:tidy"));
    let o = repo.run(
        &repo.path(),
        &["pattern", "--", "When you test: set A=1."],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    let said = claude.run(&repo, claude.start, &start);
    assert!(said.contains("no tidy has run yet"), "{said}");
    assert!(said.contains("offer them /celis:tidy"), "{said}");
    let o = repo.run(
        &repo.path(),
        &[
            "step",
            "add",
            "--new",
            "--",
            "Tidy the celis trails of this project",
        ],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    assert!(!claude
        .run(&repo, claude.start, &start)
        .contains("/celis:tidy"));
}

/// Each agent's prompt hook writes the prompt on its session's trail while a step works, in the person's
/// project (`spec/04-trails.md`, "Sessions").
#[test]
fn each_agents_prompt_hook_writes_a_prompt_on_its_sessions_trail() {
    for agent in AGENTS.into_iter().filter(|a| !a.ask.is_empty()) {
        let repo = Repo::new();
        let add = repo.run(
            &repo.path(),
            &["step", "add", "--session", "s1", "--", "the trail"],
            &[],
            "",
        );
        assert!(add.status.success(), "{}", err(&add));
        let trail = out(&add).trim().to_string();
        let start = repo.run(&repo.path(), &["step", "start", &trail[..8]], &[], "");
        assert!(start.status.success(), "{}", err(&start));
        let stdout = agent.run(
            &repo,
            agent.ask,
            &agent.payload("s1", &repo.path(), "make it blue"),
        );
        assert_eq!(
            prompts(&repo, &trail),
            ["make it blue"],
            "{}: {stdout}",
            agent.name
        );
    }
}

/// Claude Code's prompt hook asks for the patterns of the ask at each prompt, not only at the first of a
/// session.
#[test]
fn claudes_prompt_hook_asks_for_recall_at_each_prompt() {
    let repo = Repo::new();
    let o = repo.run(
        &repo.path(),
        &["pattern", "--", "When you test: set A=1."],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    let claude = &AGENTS[0];
    for prompt in ["make it blue", "now add a test"] {
        let ask = claude.payload("s1", &repo.path(), prompt);
        let said = claude.run(&repo, claude.ask, &ask);
        assert!(said.contains("celis patterns"), "{prompt}: {said}");
    }
    let o = repo.run(
        &repo.path(),
        &["step", "add", "--session", "s1", "--", "make it blue"],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    let ask = claude.payload("s1", &repo.path(), "and the border too");
    let said = claude.run(&repo, claude.ask, &ask);
    assert!(said.contains("celis patterns"), "{said}");
}

/// The prompt hooks of Claude Code, Codex and Gemini CLI give one line, with no past agent and no
/// `past.md`.
#[test]
fn each_prompt_hook_gives_one_line() {
    let repo = Repo::new();
    let o = repo.run(
        &repo.path(),
        &["pattern", "--", "When you test: set A=1."],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    for agent in [&AGENTS[0], &AGENTS[1], &AGENTS[3]] {
        let ask = agent.payload(agent.name, &repo.path(), "make it blue");
        let said = agent.run(&repo, agent.ask, &ask);
        let said = agent.context(&said);
        assert_eq!(said.trim_end().lines().count(), 1, "{}: {said}", agent.name);
        assert!(
            !said.contains("celis:past") && !said.contains("past.md"),
            "{}: {said}",
            agent.name
        );
    }
}

/// Each manifest names the hooks file the agent runs, and each marketplace names the plugin's folder.
#[test]
fn each_manifest_names_its_hooks_and_each_marketplace_the_plugin() {
    let hooks = [
        (
            "plugins/celis/.codex-plugin/plugin.json",
            "./hooks/hooks-codex.json",
        ),
        (
            "plugins/celis/.cursor-plugin/plugin.json",
            "./hooks/hooks-cursor.json",
        ),
    ];
    for (manifest, file) in hooks {
        assert_eq!(read(manifest)["hooks"], file, "{manifest}");
    }
    let codex = read(".agents/plugins/marketplace.json");
    assert_eq!(codex["plugins"][0]["source"]["path"], "./plugins/celis");
    assert_eq!(
        read(".cursor-plugin/marketplace.json")["plugins"][0]["source"],
        "plugins/celis"
    );
    assert_eq!(
        read(".claude-plugin/marketplace.json")["plugins"][0]["source"],
        "./plugins/celis"
    );
    assert_eq!(read("gemini-extension.json")["name"], "celis");
    assert_eq!(read(".antigravity-plugin/plugin.json")["name"], "celis");
}

/// Each agent's manifest carries one version, so each agent updates the plugin on a release.
#[test]
fn each_manifest_carries_one_version() {
    let manifests = [
        "plugins/celis/.claude-plugin/plugin.json",
        "plugins/celis/.codex-plugin/plugin.json",
        "plugins/celis/.cursor-plugin/plugin.json",
        "gemini-extension.json",
        ".antigravity-plugin/plugin.json",
    ];
    let first = read(manifests[0])["version"].clone();
    assert!(first.is_string(), "{}: no version", manifests[0]);
    for manifest in manifests {
        assert_eq!(read(manifest)["version"], first, "{manifest}");
    }
}

/// The Claude and Cursor manifests give their directory listings the same links and logo, and the Claude
/// manifest gives the pages its directory asks for.
#[test]
fn each_listed_manifest_carries_its_links() {
    let claude = read("plugins/celis/.claude-plugin/plugin.json");
    let cursor = read("plugins/celis/.cursor-plugin/plugin.json");
    assert_eq!(claude["homepage"], "https://celis.ai");
    assert_eq!(claude["repository"], "https://github.com/celishq/trails");
    for field in ["homepage", "repository", "license"] {
        assert_eq!(cursor[field], claude[field], "{field}");
    }
    let icon = claude["icon"].as_str().expect("icon");
    assert_eq!(cursor["logo"], icon);
    assert!(
        checkout().join("plugins/celis").join(icon).is_file(),
        "{icon}"
    );
    for (field, page) in [
        ("documentationUrl", "docs"),
        ("supportUrl", "support"),
        ("privacyPolicyUrl", "privacy"),
        ("termsOfServiceUrl", "terms"),
    ] {
        assert_eq!(claude[field], format!("https://celis.ai/{page}"), "{field}");
    }
}

/// With no `celis` on PATH, each agent's session start says to install it from celis.ai, and each prompt
/// hook prints nothing.
#[test]
fn each_agents_hooks_say_how_to_install_a_missing_celis() {
    let repo = Repo::new();
    let empty = tempfile::tempdir().unwrap();
    for agent in AGENTS {
        let stdout = agent.run_from(
            &repo,
            agent.start,
            &agent.payload("s1", &repo.path(), ""),
            empty.path(),
        );
        let context = agent.context(&stdout);
        assert!(
            context.contains("install it from https://celis.ai"),
            "{}: {context}",
            agent.name
        );
        if !agent.ask.is_empty() {
            let payload = agent.payload("s1", &repo.path(), "make it blue");
            let said = agent.run_from(&repo, agent.ask, &payload, empty.path());
            assert_eq!(said, "", "{}", agent.name);
        }
    }
}

/// Claude Code's session start says to update a `celis` that has no `hook` command.
#[test]
fn claudes_session_start_says_to_update_an_older_celis() {
    let repo = Repo::new();
    let bin = tempfile::tempdir().unwrap();
    let old = bin.path().join("celis");
    std::fs::write(
        &old,
        "#!/bin/sh\necho 'celis: no command \"hook\"' >&2\nexit 2\n",
    )
    .unwrap();
    Command::new("chmod").arg("755").arg(&old).status().unwrap();
    let claude = &AGENTS[0];
    let said = claude.run_from(
        &repo,
        claude.start,
        &claude.payload("s1", &repo.path(), ""),
        bin.path(),
    );
    assert!(said.contains("run: celis update"), "{said}");
}

/// Claude Code's hooks name each script by a literal path under `${CLAUDE_PLUGIN_ROOT}`, and each script
/// uses no variable but `CLAUDE_PLUGIN_ROOT` and no substitution, so the plugin directory follows them.
#[test]
fn claudes_hook_scripts_compute_no_path() {
    let claude = &AGENTS[0];
    for (event, script) in [(claude.start, "start.sh"), (claude.ask, "ask.sh")] {
        let named = format!("\"${{CLAUDE_PLUGIN_ROOT}}/hooks/{script}\"");
        assert_eq!(claude.command(event), named);
        let text =
            std::fs::read_to_string(checkout().join("plugins/celis/hooks").join(script)).unwrap();
        for line in text.lines().filter(|l| !l.starts_with('#')) {
            assert!(
                !line.contains("$(") && !line.contains('`'),
                "{script}: {line}"
            );
            let rest: Vec<&str> = line.split('$').skip(1).collect();
            assert!(
                rest.iter().all(|r| r.starts_with("CLAUDE_PLUGIN_ROOT\"")),
                "{script}: {line}"
            );
        }
    }
}
