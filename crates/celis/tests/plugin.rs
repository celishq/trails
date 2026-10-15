//! The plugin's hooks, run as each agent's manifest declares them (CIP-04, "Sessions").

mod support;

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
        let root = checkout().join(self.root.1);
        let variable = Some(self.root.0).filter(|v| !v.is_empty());
        let celis = Path::new(env!("CARGO_BIN_EXE_celis")).parent().unwrap();
        let folder = if self.in_root {
            root.clone()
        } else {
            repo.path()
        };
        let mut child = Command::new("sh")
            .args(["-c", &self.command(event)])
            .current_dir(folder)
            .env_clear()
            .env("PATH", format!("{}:/usr/bin:/bin", celis.display()))
            .env("CELIS_HOME", repo.home())
            .envs(variable.map(|v| (v, &root)))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        std::io::Write::write_all(&mut child.stdin.take().unwrap(), stdin.as_bytes()).unwrap();
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
/// pass with `--session`. Antigravity has no session start, and gets both before each call of the model
/// (`spec/04-trails.md`, "Sessions").
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
    }
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
        let lines: Vec<String> = out(&add).lines().map(str::to_string).collect();
        let (step, trail) = (&lines[0], &lines[1]);
        let start = repo.run(&repo.path(), &["step", "start", &step[..8]], &[], "");
        assert!(start.status.success(), "{}", err(&start));
        let stdout = agent.run(
            &repo,
            agent.ask,
            &agent.payload("s1", &repo.path(), "make it blue"),
        );
        assert_eq!(
            prompts(&repo, trail),
            ["make it blue"],
            "{}: {stdout}",
            agent.name
        );
    }
}

/// Claude Code's prompt hook asks for the past agent once a session, and again after a compaction's
/// session start (`spec/02-cli.md`, "Prompt").
#[test]
fn claudes_hooks_ask_for_the_past_agent_again_after_a_compaction() {
    let repo = Repo::new();
    let o = repo.run(
        &repo.path(),
        &["decide", "--", "When you test: set A=1."],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    let claude = &AGENTS[0];
    let ask = claude.payload("s1", &repo.path(), "make it blue");
    assert!(claude
        .run(&repo, claude.ask, &ask)
        .contains("celis:past agent"));
    assert!(!claude
        .run(&repo, claude.ask, &ask)
        .contains("celis:past agent"));
    let compact = serde_json::json!({"session_id": "s1", "cwd": repo.path(), "source": "compact"});
    claude.run(&repo, claude.start, &compact.to_string());
    assert!(claude
        .run(&repo, claude.ask, &ask)
        .contains("celis:past agent"));
}

/// Codex's prompt hook asks to do what past.md says in place of the past agent it does not have
/// (`spec/04-trails.md`, "The past").
#[test]
fn codexs_prompt_hook_names_past_md_for_the_past_agent() {
    let repo = Repo::new();
    let o = repo.run(
        &repo.path(),
        &["decide", "--", "When you test: set A=1."],
        &[],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    let codex = &AGENTS[1];
    let said = codex.run(
        &repo,
        codex.ask,
        &codex.payload("s1", &repo.path(), "make it blue"),
    );
    assert!(said.contains("do what past.md says"), "{said}");
    assert!(!said.contains("celis:past"), "{said}");
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

/// Each agent's manifest carries one version, so each agent updates the plugin on a release
/// (`spec/04-trails.md`, "Plugin version").
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

/// What Claude Code's session start gives when `celis --version` prints `celis` and the manifest carries
/// `plugin`, run from a copy of the plugin with a `celis` that only prints its version.
fn start_with_versions(celis: &str, plugin: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("celis");
    for rel in [
        "hooks/start.sh",
        "hooks/out.sh",
        "card.md",
        ".claude-plugin/plugin.json",
    ] {
        let to = root.join(rel);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(checkout().join("plugins/celis").join(rel), &to).unwrap();
    }
    let manifest = serde_json::json!({"name": "celis", "version": plugin});
    std::fs::write(
        root.join(".claude-plugin/plugin.json"),
        manifest.to_string(),
    )
    .unwrap();
    let bin = dir.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let fake =
        format!("#!/bin/sh\n[ \"$1\" = --version ] && echo 'celis {celis} (abcd1234)'\nexit 0\n");
    std::fs::write(bin.join("celis"), fake).unwrap();
    Command::new("chmod")
        .arg("755")
        .arg(bin.join("celis"))
        .status()
        .unwrap();
    let o = Command::new("sh")
        .arg(root.join("hooks/start.sh"))
        .env_clear()
        .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", err(&o));
    out(&o)
}

/// The session start names both versions when the plugin and `celis` differ, and nothing when they match
/// or one is a build from the repo (`spec/04-trails.md`, "Plugin version").
#[test]
fn session_start_names_a_version_mismatch() {
    let notice = "celis 1.4.0 is installed, and the celis plugin is 1.3.0.";
    assert!(start_with_versions("1.4.0", "1.3.0").contains(notice));
    for (celis, plugin) in [("1.4.0", "1.4.0"), ("0.0.0", "1.4.0"), ("1.4.0", "0.0.0")] {
        let context = start_with_versions(celis, plugin);
        assert!(context.starts_with("How to do an ask"), "{context}");
        assert!(
            !context.contains("is installed, and the celis plugin"),
            "{celis} {plugin}: {context}"
        );
    }
}
