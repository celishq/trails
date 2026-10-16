//! A git repo in a temp folder, a `$CELIS_HOME` with the fixture's account, and the binary run in them.
#![allow(dead_code)]

use celis_nostr::{Account, Note};
use celis_trails::{Project, Store};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

pub mod watch;

/// The phrase of every test home.
pub const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// A git repo with one commit, and a `$CELIS_HOME` beside it whose phrase is [`PHRASE`].
pub struct Repo {
    dir: TempDir,
}

impl Repo {
    pub fn new() -> Repo {
        let dir = tempfile::tempdir().expect("a temp folder");
        let repo = Repo { dir };
        std::fs::create_dir_all(repo.path()).unwrap();
        repo.git(&["init", "-q", "-b", "main"]);
        repo.write("README.md", "a repo\n");
        repo.git(&["add", "."]);
        repo.git(&[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "-q",
            "-m",
            "first",
        ]);
        std::fs::create_dir_all(repo.home()).unwrap();
        std::fs::write(repo.home().join("phrase"), format!("{PHRASE}\n")).unwrap();
        repo
    }

    /// A folder outside git, with a `$CELIS_HOME` beside it whose phrase is [`PHRASE`].
    pub fn plain() -> Repo {
        let root = std::fs::canonicalize(std::env::temp_dir()).unwrap();
        let dir = tempfile::tempdir_in(root).expect("a temp folder");
        let repo = Repo { dir };
        std::fs::create_dir_all(repo.path()).unwrap();
        std::fs::create_dir_all(repo.home()).unwrap();
        std::fs::write(repo.home().join("phrase"), format!("{PHRASE}\n")).unwrap();
        repo
    }

    /// The main checkout.
    pub fn path(&self) -> PathBuf {
        self.dir.path().join("repo")
    }

    /// The `$CELIS_HOME` the commands use.
    pub fn home(&self) -> PathBuf {
        self.dir.path().join("home")
    }

    /// The store every command opens.
    pub fn store(&self) -> PathBuf {
        self.home().join("trails")
    }

    /// The id of the main checkout's project: its first commit.
    pub fn project_id(&self) -> String {
        Project::at(&self.path()).id
    }

    /// A new worktree on a new branch `name`, beside the main checkout.
    pub fn worktree(&self, name: &str) -> PathBuf {
        let path = self.dir.path().join(name);
        self.git(&["worktree", "add", "-q", "-b", name, path.to_str().unwrap()]);
        path
    }

    /// A second git repo `name`, with one commit, beside the main checkout; it shares this home.
    pub fn other(&self, name: &str) -> PathBuf {
        let path = self.dir.path().join(name);
        std::fs::create_dir_all(&path).unwrap();
        let commit = ["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q"];
        let mut first = commit.to_vec();
        first.extend(["--allow-empty", "-m", name]);
        for args in [vec!["init", "-q"], first] {
            let run = Command::new("git").args(args).current_dir(&path).status();
            assert!(run.unwrap().success());
        }
        path
    }

    /// Writes `text` to `rel` in the main checkout.
    pub fn write(&self, rel: &str, text: &str) {
        let path = self.path().join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    pub fn git(&self, args: &[&str]) -> String {
        let out = Command::new("git")
            .args(args)
            .current_dir(self.path())
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// Runs `celis` in `folder` with `args`, this repo's home, the variables `env`, and `stdin`.
    pub fn run(&self, folder: &Path, args: &[&str], env: &[(&str, &str)], stdin: &str) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_celis"))
            .args(args)
            .current_dir(folder)
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .env("CELIS_HOME", self.home())
            .envs(env.iter().copied())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(stdin.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }
}

pub const DAY: u64 = 86_400;

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn shared(repo: &Repo) -> std::path::PathBuf {
    let path = repo.store().join("shared");
    std::fs::create_dir_all(&path).unwrap();
    path
}

pub fn put(repo: &Repo, note: &Note) {
    Store::open(&repo.home()).unwrap().write(note).unwrap();
}

/// Writes an old steer record with `text` on the step `id`.
pub fn steer(repo: &Repo, id: &str, text: &str) {
    let tags: Vec<Vec<String>> = vec![
        vec!["L".into(), "celis.step".into()],
        vec!["l".into(), "steer".into(), "celis.step".into()],
        vec!["e".into(), id.into()],
    ];
    put(
        repo,
        &Note::write(&account(), 1985u16, now(), tags, text).unwrap(),
    );
}

/// A cairn on the repo that `by` signed at `at`, with the extra tags `extra`.
pub fn cairn(repo: &Repo, by: &Account, label: &str, extra: &[(&str, &str)], at: u64) -> Note {
    let anchor = format!("project:{}", repo.project_id());
    let mut tags: Vec<Vec<String>> = vec![
        vec!["I".into(), anchor.clone()],
        vec!["K".into(), "project".into()],
        vec!["i".into(), anchor],
        vec!["k".into(), "project".into()],
        vec!["project".into(), "repo".into()],
        vec!["L".into(), "celis.cairn".into()],
        vec!["l".into(), label.into(), "celis.cairn".into()],
    ];
    for (name, value) in extra {
        tags.push(vec![name.to_string(), value.to_string()]);
    }
    let note = Note::write(by, 1111u16, at, tags, "text").unwrap();
    put(repo, &note);
    note
}

/// `by`'s rating of `cairn`, written at `at`.
pub fn rating(repo: &Repo, by: &Account, cairn: &Note, helped: bool, at: u64) {
    let tags = vec![
        vec!["e".to_string(), cairn.id.clone()],
        vec!["p".to_string(), cairn.pubkey.hex()],
        vec!["k".to_string(), "1111".to_string()],
    ];
    let content = if helped { "+" } else { "-" };
    put(repo, &Note::write(by, 7u16, at, tags, content).unwrap());
}

/// Every cairn record in the store with the label `label`.
pub fn labelled(repo: &Repo, label: &str) -> Vec<Note> {
    std::fs::read_dir(shared(repo))
        .unwrap()
        .filter_map(|e| std::fs::read_to_string(e.unwrap().path()).ok())
        .filter_map(|t| serde_json::from_str::<Note>(&t).ok())
        .filter(|n| n.kind == 1111 && n.tag("l") == Some(label))
        .collect()
}

/// The account [`PHRASE`] gives.
pub fn account() -> Account {
    celis_nostr::Identity::from_phrase(PHRASE)
        .unwrap()
        .account()
        .clone()
}

/// What `o` wrote to stdout.
pub fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

/// What `o` wrote to stderr.
pub fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

/// Two records labelled `first` and `second`, signed by `by`, whose ids share their first 8
/// characters. Nothing is written to the store.
pub fn sharing_a_start(repo: &Repo, by: &Account, first: &str, second: &str) -> (Note, Note) {
    let project_id = repo.project_id();
    let at = now() - DAY;
    let pubkey = by.pubkey().hex();
    let tags = |label: &str| {
        let anchor = format!("project:{project_id}");
        vec![
            vec!["I".to_string(), anchor.clone()],
            vec!["K".into(), "project".into()],
            vec!["i".into(), anchor],
            vec!["k".into(), "project".into()],
            vec!["project".into(), "repo".into()],
            vec!["L".into(), "celis.step".into()],
            vec!["l".into(), label.into(), "celis.step".into()],
        ]
    };
    let tags_of = [tags(first), tags(second)];
    let id_of = |side: usize, n: u32| {
        let body = serde_json::json!([0, pubkey, at, 1111, tags_of[side], format!("note {n}")]);
        format!("{:x}", Sha256::digest(body.to_string().as_bytes()))
    };
    let write = |side: usize, n: u32, id: &str| {
        let note =
            Note::write(by, 1111u16, at, tags_of[side].clone(), format!("note {n}")).unwrap();
        assert_eq!(note.id, id, "the id hashed by hand is the id written");
        note
    };
    let mut seen: HashMap<String, (usize, u32, String)> = HashMap::new();
    for n in 0.. {
        for side in [0, 1] {
            let id = id_of(side, n);
            let start = id[..8].to_string();
            if let Some((other, m, other_id)) = seen.get(&start) {
                if (first == second || *other != side) && *other_id != id {
                    let (m, other_id) = (*m, other_id.clone());
                    let (s, t) = (*other, side);
                    return (write(s, m, &other_id), write(t, n, &id));
                }
            }
            seen.entry(start).or_insert((side, n, id));
            if first == second {
                break;
            }
        }
    }
    unreachable!()
}

/// Asserts that `o` exited with `code` and printed the error `name` as a JSON object on stderr.
pub fn refusal(o: &Output, code: i32, name: &str) {
    assert_eq!(o.status.code(), Some(code), "{}", err(o));
    let object: serde_json::Value =
        serde_json::from_str(err(o).trim()).unwrap_or_else(|_| panic!("{}", err(o)));
    assert_eq!(object["error"], name, "{}", err(o));
}

/// A decision of `by` on `project` (id, name) saying `content`, with the tags `scope` and `replaces`
/// when given, written at `created`, put in the store.
pub fn decision(
    repo: &Repo,
    project: (&str, &str),
    by: &Account,
    place: (Option<&str>, Option<&str>),
    content: &str,
    created: u64,
) -> Note {
    let anchor = format!("project:{}", project.0);
    let mut tags: Vec<Vec<String>> = vec![
        vec!["I".into(), anchor.clone()],
        vec!["K".into(), "project".into()],
        vec!["i".into(), anchor],
        vec!["k".into(), "project".into()],
        vec!["project".into(), project.1.into()],
        vec!["L".into(), "celis.decision".into()],
        vec!["l".into(), "decision".into(), "celis.decision".into()],
    ];
    if let Some(scope) = place.0 {
        tags.push(vec!["scope".into(), scope.into()]);
    }
    if let Some(id) = place.1 {
        tags.push(vec!["replaces".into(), id.into()]);
    }
    let note = Note::write(by, 1111u16, created, tags, content).unwrap();
    put(repo, &note);
    note
}

/// A step of this repo under `under`, when given, that `by` signed at `at`, saying `content`, put in the
/// store.
pub fn step_note(repo: &Repo, by: &Account, under: Option<&Note>, content: &str, at: u64) -> Note {
    let mut tags = step_tags((&repo.project_id(), "repo"));
    tags.extend(under.map(|u| vec!["under".to_string(), u.id.clone()]));
    let note = Note::write(by, 1111u16, at, tags, content).unwrap();
    put(repo, &note);
    note
}

/// A `done` of `step` that `by` signed at `at`, put in the store.
pub fn done_note(repo: &Repo, by: &Account, step: &Note, at: u64) -> Note {
    let tags: Vec<Vec<String>> = vec![
        vec!["L".into(), "celis.step".into()],
        vec!["l".into(), "done".into(), "celis.step".into()],
        vec!["e".into(), step.id.clone()],
    ];
    let note = Note::write(by, 1985u16, at, tags, "proof").unwrap();
    put(repo, &note);
    note
}

/// An open step of this repo that `by` signed at `at`, saying `content`, put in the store.
pub fn step_signed(repo: &Repo, by: &Account, content: &str, at: u64) -> Note {
    let anchor = format!("project:{}", repo.project_id());
    let tags: Vec<Vec<String>> = vec![
        vec!["I".into(), anchor.clone()],
        vec!["K".into(), "project".into()],
        vec!["i".into(), anchor],
        vec!["k".into(), "project".into()],
        vec!["project".into(), "repo".into()],
        vec!["L".into(), "celis.step".into()],
        vec!["l".into(), "step".into(), "celis.step".into()],
    ];
    let note = Note::write(by, 1111u16, at, tags, content).unwrap();
    put(repo, &note);
    note
}

/// Two distinct kind 1111 records of `by` at `at` with `tags`, whose ids share their first 8 characters.
/// Nothing is written to the store.
pub fn sharing_a_start_with(by: &Account, at: u64, tags: &[Vec<String>]) -> (Note, Note) {
    let pubkey = by.pubkey().hex();
    let mut seen: HashMap<String, u32> = HashMap::new();
    for n in 0u32.. {
        let body = serde_json::json!([0, pubkey, at, 1111, tags, format!("note {n}")]);
        let id = format!("{:x}", Sha256::digest(body.to_string().as_bytes()));
        if let Some(m) = seen.insert(id[..8].to_string(), n) {
            let write = |k: u32| Note::write(by, 1111u16, at, tags.to_vec(), format!("note {k}"));
            return (write(m).unwrap(), write(n).unwrap());
        }
    }
    unreachable!()
}

/// The tags of a step on `project` (id, name).
pub fn step_tags(project: (&str, &str)) -> Vec<Vec<String>> {
    let mut tags = decision_tags(project);
    tags.truncate(5);
    tags.push(vec!["L".into(), "celis.step".into()]);
    tags.push(vec!["l".into(), "step".into(), "celis.step".into()]);
    tags
}

/// The tags of a decision on `project` (id, name).
pub fn decision_tags(project: (&str, &str)) -> Vec<Vec<String>> {
    let anchor = format!("project:{}", project.0);
    vec![
        vec!["I".into(), anchor.clone()],
        vec!["K".into(), "project".into()],
        vec!["i".into(), anchor],
        vec!["k".into(), "project".into()],
        vec!["project".into(), project.1.into()],
        vec!["L".into(), "celis.decision".into()],
        vec!["l".into(), "decision".into(), "celis.decision".into()],
    ]
}

/// `celis <args>` run in the main checkout.
pub fn cli(repo: &Repo, args: &[&str]) -> Output {
    repo.run(&repo.path(), args, &[], "")
}

/// The id `celis step add <args>` prints in the main checkout.
pub fn add(repo: &Repo, args: &[&str]) -> String {
    let mut all = vec!["step", "add"];
    all.extend(args);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).trim().to_string()
}

/// Closes the step `id` with a proof, and asserts it closed.
pub fn finish(repo: &Repo, id: &str) {
    let o = cli(
        repo,
        &[
            "step",
            "done",
            id,
            "--as-planned",
            "--",
            "failed before, passes now",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
}

/// The JSON objects `celis <args> --json` prints in the main checkout, one a line.
pub fn objects(repo: &Repo, args: &[&str]) -> Vec<serde_json::Value> {
    let mut all = args.to_vec();
    all.push("--json");
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o)
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// The ids, first 8 characters, of the decisions `step start` prints in `folder` for a new step that
/// `step add --new <args>` adds, in order.
pub fn read_ids(repo: &Repo, folder: &Path, args: &[&str]) -> Vec<String> {
    let text = format!("a step that reads {}", args.join(" "));
    let mut all = vec!["step", "add", "--new"];
    all.extend(args);
    all.extend(["--", &text]);
    let o = repo.run(folder, &all, &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let id = out(&o).lines().next().unwrap().to_string();
    let o = repo.run(folder, &["step", "start", &id, "--json"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    out(&o)
        .lines()
        .filter_map(|l| {
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            Some(v.get("decision")?["id"].as_str()?[..8].to_string())
        })
        .collect()
}

/// The ids, first 8 characters, of the live rules of the project `name` that `celis grep` prints in
/// `folder`, in order.
pub fn live_ids(repo: &Repo, folder: &Path, name: &str) -> Vec<String> {
    let o = repo.run(folder, &["grep", " rule [0-9a-f]* live "], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let project = format!("[{name}]");
    found(&out(&o))
        .into_iter()
        .filter(|(of, line)| *of == project && line.split(' ').nth(2) == Some("live"))
        .filter_map(|(_, line)| Some(line.strip_prefix("rule ")?.split(' ').next()?.to_string()))
        .collect()
}

/// `[<project>]` and the line from its kind on, of each line `celis grep` printed in `text`.
pub fn found(text: &str) -> Vec<(String, String)> {
    let mut project = String::new();
    let mut lines = Vec::new();
    for l in text.lines().take_while(|l| !l.is_empty()) {
        let Some(line) = l.strip_prefix("  ") else {
            project = l.split(' ').nth(1).unwrap_or("").to_string();
            continue;
        };
        lines.push(match line.split_once(' ') {
            Some((of, rest)) if of.starts_with('[') => (of.to_string(), rest.to_string()),
            _ => (project.clone(), line.to_string()),
        });
    }
    lines
}

/// The ids, first 8 characters, of the lines of `kind` that `celis grep` printed in `text`, in order.
pub fn ids(text: &str, kind: &str) -> Vec<String> {
    let kind = format!("{kind} ");
    found(text)
        .into_iter()
        .filter_map(|(_, line)| Some(line.strip_prefix(&kind)?.split(' ').next()?.to_string()))
        .collect()
}
