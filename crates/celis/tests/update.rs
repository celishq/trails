//! CIP-02, "Update".

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

/// A release server on localhost: the path of each answer, and the paths asked so far.
struct Server {
    base: String,
    asked: Arc<Mutex<Vec<String>>>,
}

impl Server {
    /// Serves `latest` as the newest release, and each of `releases` as an archive of a `celis` that
    /// prints `celis <version>`; the checksum of `wrong` does not match.
    fn new(latest: &str, releases: &[&str], wrong: Option<&str>) -> Server {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}/releases", listener.local_addr().unwrap());
        let mut answers: HashMap<String, (String, Vec<u8>)> = HashMap::new();
        answers.insert(
            "/releases/latest".to_string(),
            (
                format!("302 Found\r\nLocation: {base}/tag/v{latest}"),
                vec![],
            ),
        );
        for version in releases {
            let archive = archive(version);
            let mut sum = format!("{:x}", Sha256::digest(&archive));
            if wrong == Some(*version) {
                sum = "0".repeat(64);
            }
            let path = format!("/releases/download/v{version}/{}", name());
            answers.insert(
                format!("{path}.sha256"),
                ("200 OK".to_string(), format!("{sum}  {}\n", name()).into()),
            );
            answers.insert(path, ("200 OK".to_string(), archive));
        }
        let asked = Arc::new(Mutex::new(vec![]));
        let seen = asked.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut line = String::new();
                let mut reader = BufReader::new(&stream);
                reader.read_line(&mut line).unwrap();
                while reader.read_line(&mut String::new()).unwrap() > 2 {}
                let path = line.split(' ').nth(1).unwrap_or("").to_string();
                seen.lock().unwrap().push(path.clone());
                let (status, body) = answers
                    .get(&path)
                    .cloned()
                    .unwrap_or(("404 Not Found".to_string(), vec![]));
                let head = format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&body);
            }
        });
        Server { base, asked }
    }

    /// Whether a download of an archive was asked.
    fn downloaded(&self) -> bool {
        self.asked
            .lock()
            .unwrap()
            .iter()
            .any(|p| p.contains("/download/"))
    }
}

/// The archive name of this system and processor.
fn name() -> String {
    let system = match std::env::consts::OS {
        "macos" => "apple-darwin",
        _ => "unknown-linux-musl",
    };
    format!("celis-{}-{system}.tar.gz", std::env::consts::ARCH)
}

/// A `.tar.gz` that holds one `celis`, a script that prints `celis <version>`.
fn archive(version: &str) -> Vec<u8> {
    let script = format!("#!/bin/sh\necho \"celis {version}\"\n");
    let mut header = tar::Header::new_gnu();
    header.set_size(script.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    let gz = flate2::write::GzEncoder::new(vec![], flate2::Compression::default());
    let mut builder = tar::Builder::new(gz);
    builder
        .append_data(&mut header, "celis", script.as_bytes())
        .unwrap();
    builder.into_inner().unwrap().finish().unwrap()
}

/// A copy of the built `celis` in a folder of its own, as an install puts it.
struct Install {
    folder: tempfile::TempDir,
}

impl Install {
    fn new() -> Install {
        let folder = tempfile::tempdir().unwrap();
        std::fs::create_dir(folder.path().join("bin")).unwrap();
        std::fs::copy(env!("CARGO_BIN_EXE_celis"), folder.path().join("bin/celis")).unwrap();
        Install { folder }
    }

    fn file(&self) -> PathBuf {
        self.folder.path().join("bin/celis")
    }

    /// Runs `celis <args>` from `file` against `server`.
    fn run(&self, file: &Path, server: &Server, args: &[&str]) -> Output {
        Command::new(file)
            .args(args)
            .env("CELIS_BASE", &server.base)
            .env("CELIS_HOME", self.folder.path().join("home"))
            .output()
            .unwrap()
    }

    /// `celis <version>` as `celis --version` of the installed file prints it, without the commit.
    fn version(&self) -> String {
        let o = Command::new(self.file()).arg("--version").output().unwrap();
        let shown = String::from_utf8_lossy(&o.stdout);
        shown
            .split(' ')
            .take(2)
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }
}

fn out(o: &Output) -> String {
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

/// `update` installs the release `latest` redirects to over the file the running `celis` resolves
/// to (`spec/trails/02-cli.md`, "Update").
#[test]
fn update_installs_the_latest_release_over_the_resolved_file() {
    let install = Install::new();
    let server = Server::new("9.9.9", &["9.9.9"], None);
    let link = install.folder.path().join("celis");
    std::os::unix::fs::symlink(install.file(), &link).unwrap();
    let shown = out(&install.run(&link, &server, &["update"]));
    let path = install.file().canonicalize().unwrap();
    assert_eq!(
        shown,
        format!("Updated celis 0.0.0 to 9.9.9 at {}.", path.display())
    );
    assert_eq!(install.version(), "celis 9.9.9");
    assert!(link.is_symlink());
}

/// When the latest version is installed, `update` downloads nothing and says so
/// (`spec/trails/02-cli.md`, "Update").
#[test]
fn update_downloads_nothing_when_the_latest_is_installed() {
    let install = Install::new();
    let server = Server::new("0.0.0", &["0.0.0"], None);
    let shown = out(&install.run(&install.file(), &server, &["update"]));
    assert_eq!(shown, "celis 0.0.0 is the latest.");
    assert!(!server.downloaded());
}

/// With a version, `update` installs that version, though it is not the latest
/// (`spec/trails/02-cli.md`, "Update").
#[test]
fn update_installs_a_named_version() {
    let install = Install::new();
    let server = Server::new("9.9.9", &["9.9.9", "1.2.0"], None);
    let o = install.run(&install.file(), &server, &["update", "1.2.0", "--json"]);
    let shown: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    let path = install.file().canonicalize().unwrap();
    assert_eq!(
        shown,
        serde_json::json!({"from": "0.0.0", "to": "1.2.0", "path": path, "updated": true})
    );
    assert_eq!(install.version(), "celis 1.2.0");
}

/// With `--dry-run`, `update` prints the installed and the new version, and changes nothing
/// (`spec/trails/02-cli.md`, "Update").
#[test]
fn update_dry_run_changes_nothing() {
    let install = Install::new();
    let server = Server::new("9.9.9", &["9.9.9"], None);
    let shown = out(&install.run(&install.file(), &server, &["update", "--dry-run"]));
    assert_eq!(
        shown,
        "celis 0.0.0 is installed; celis update installs 9.9.9."
    );
    assert!(!server.downloaded());
    assert_eq!(install.version(), "celis 0.0.0");
}

/// When the archive does not match its checksum, `update` fails and leaves the installed file as it
/// was (`spec/trails/02-cli.md`, "Update").
#[test]
fn update_fails_on_a_wrong_checksum_and_keeps_the_file() {
    let install = Install::new();
    let server = Server::new("9.9.9", &["9.9.9"], Some("9.9.9"));
    let o = install.run(&install.file(), &server, &["update"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("checksum"),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert_eq!(install.version(), "celis 0.0.0");
    let left: Vec<_> = std::fs::read_dir(install.folder.path().join("bin"))
        .unwrap()
        .collect();
    assert_eq!(left.len(), 1);
}
