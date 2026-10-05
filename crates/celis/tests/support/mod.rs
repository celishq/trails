//! A git repo in a temp folder, a `$CELIS_HOME` with the fixture's account, and the binary run in them.
#![allow(dead_code)]

use celis_cairns::{Agent, Silo};
use celis_wire::{Account, Note};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

/// The account secret `spec/fixtures/cairn.json` holds.
pub const SECRET: &str = "0101010101010101010101010101010101010101010101010101010101010101";

/// A git repo with one commit, and a `$CELIS_HOME` beside it whose key is [`SECRET`].
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
        std::fs::write(repo.home().join("key"), format!("{SECRET}\n")).unwrap();
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

    /// The silo the main checkout opens.
    pub fn silo(&self) -> PathBuf {
        self.path().join(".celis/cairns")
    }

    /// The 32-hex id in the silo's `id` file; opens the silo first, so the file exists.
    pub fn silo_id(&self) -> String {
        Silo::open(&self.path(), Some(&self.home())).unwrap();
        std::fs::read_to_string(self.silo().join("id"))
            .unwrap()
            .trim()
            .to_string()
    }

    /// A new worktree on a new branch `name`, beside the main checkout.
    pub fn worktree(&self, name: &str) -> PathBuf {
        let path = self.dir.path().join(name);
        self.git(&["worktree", "add", "-q", "-b", name, path.to_str().unwrap()]);
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

    /// Runs `celis cairn` in `folder` with `args`, this repo's home, the variables `env`, and `stdin`.
    pub fn run(&self, folder: &Path, args: &[&str], env: &[(&str, &str)], stdin: &str) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_celis"))
            .arg("cairn")
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
    let path = repo.silo().join("shared");
    std::fs::create_dir_all(&path).unwrap();
    path
}

pub fn put(repo: &Repo, note: &Note) {
    Silo::open(&repo.path(), Some(&repo.home()))
        .unwrap()
        .write(note)
        .unwrap();
}

/// The agent of `session`, with its profile and link already in the silo.
pub fn agent(repo: &Repo, session: &str) -> Agent {
    let account = Account::new(&[1u8; 32]).unwrap();
    let agent = Agent::new(&account, "claude-code", session).unwrap();
    put(repo, &agent.profile("", now() - 30 * DAY).unwrap());
    put(repo, &agent.link(&account, now() - 30 * DAY).unwrap());
    agent
}

/// A cairn on the repo that `by` signed at `at`, with the extra tags `extra`.
pub fn cairn(repo: &Repo, by: &Agent, label: &str, extra: &[(&str, &str)], at: u64) -> Note {
    let anchor = format!("silo:{}", repo.silo_id());
    let mut tags: Vec<Vec<String>> = vec![
        vec!["I".into(), anchor.clone()],
        vec!["K".into(), "silo".into()],
        vec!["i".into(), anchor],
        vec!["k".into(), "silo".into()],
        vec!["L".into(), "celis.cairn".into()],
        vec!["l".into(), label.into(), "celis.cairn".into()],
    ];
    for (name, value) in extra {
        tags.push(vec![name.to_string(), value.to_string()]);
    }
    let note = Note::write(by.key(), 1111u16, at, tags, "text").unwrap();
    put(repo, &note);
    note
}

/// A finding saying `content`, written by `by` at `at`, with no `else`.
pub fn cairn_saying(repo: &Repo, by: &Agent, content: &str, at: u64) -> Note {
    let anchor = format!("silo:{}", repo.silo_id());
    let tags: Vec<Vec<String>> = vec![
        vec!["I".into(), anchor.clone()],
        vec!["K".into(), "silo".into()],
        vec!["i".into(), anchor],
        vec!["k".into(), "silo".into()],
        vec!["L".into(), "celis.cairn".into()],
        vec!["l".into(), "finding".into(), "celis.cairn".into()],
    ];
    let note = Note::write(by.key(), 1111u16, at, tags, content).unwrap();
    put(repo, &note);
    note
}

/// A finding that prevents `E-NET`.
pub fn finding(repo: &Repo, by: &Agent, at: u64) -> Note {
    let extra = [("else", "E-NET")];
    cairn(repo, by, "finding", &extra, at)
}

/// `by`'s rating of `cairn`, written at `at`.
pub fn rating(repo: &Repo, by: &Agent, cairn: &Note, helped: bool, at: u64) {
    let tags = vec![
        vec!["e".to_string(), cairn.id.clone()],
        vec!["p".to_string(), cairn.pubkey.hex()],
        vec!["k".to_string(), "1111".to_string()],
    ];
    let content = if helped { "+" } else { "-" };
    put(
        repo,
        &Note::write(by.key(), 7u16, at, tags, content).unwrap(),
    );
}

/// `celis cairn <args>` run as `session`; its stdout when it exits 0.
pub fn run(repo: &Repo, session: &str, args: &[&str]) -> String {
    let out = repo.run(&repo.path(), args, &[("CELIS_SESSION", session)], "");
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// The objects `list --json` prints with `filters`, in order.
pub fn listed(repo: &Repo, session: &str, filters: &[&str]) -> Vec<serde_json::Value> {
    let mut args = vec!["list", "--json"];
    args.extend(filters);
    run(repo, session, &args)
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// The strength `list --json` shows for `cairn`; `None` when it is not listed.
pub fn strength(repo: &Repo, cairn: &Note) -> Option<i64> {
    listed(repo, "reader", &[])
        .iter()
        .find(|o| o["id"] == cairn.id.as_str())
        .map(|o| o["strength"].as_i64().unwrap())
}

/// Every cairn record in the silo with the label `label`.
pub fn labelled(repo: &Repo, label: &str) -> Vec<Note> {
    std::fs::read_dir(shared(repo))
        .unwrap()
        .filter_map(|e| std::fs::read_to_string(e.unwrap().path()).ok())
        .filter_map(|t| serde_json::from_str::<Note>(&t).ok())
        .filter(|n| n.kind == 1111 && n.tag("l") == Some(label))
        .collect()
}

/// [`SECRET`] as an account.
pub fn account() -> Account {
    let bytes: Vec<u8> = (0..32)
        .map(|i| u8::from_str_radix(&SECRET[i * 2..i * 2 + 2], 16).unwrap())
        .collect();
    Account::new(&bytes.try_into().unwrap()).unwrap()
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
/// characters. Nothing is written to the silo.
pub fn sharing_a_start(repo: &Repo, by: &Agent, first: &str, second: &str) -> (Note, Note) {
    let silo_id = repo.silo_id();
    let at = now() - DAY;
    let pubkey = by.key().pubkey().hex();
    let tags = |label: &str| {
        let anchor = format!("silo:{silo_id}");
        vec![
            vec!["I".to_string(), anchor.clone()],
            vec!["K".into(), "silo".into()],
            vec!["i".into(), anchor],
            vec!["k".into(), "silo".into()],
            vec!["L".into(), "celis.cairn".into()],
            vec!["l".into(), label.into(), "celis.cairn".into()],
            vec!["else".into(), "E-NET".into()],
        ]
    };
    let tags_of = [tags(first), tags(second)];
    let id_of = |side: usize, n: u32| {
        let body = serde_json::json!([0, pubkey, at, 1111, tags_of[side], format!("note {n}")]);
        format!("{:x}", Sha256::digest(body.to_string().as_bytes()))
    };
    let write = |side: usize, n: u32, id: &str| {
        let note = Note::write(
            by.key(),
            1111u16,
            at,
            tags_of[side].clone(),
            format!("note {n}"),
        )
        .unwrap();
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
