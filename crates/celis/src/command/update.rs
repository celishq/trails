//! `celis update`: the running `celis`, replaced with the build of a release.

use super::{Args, Command};
use anyhow::anyhow;
use celis_trails::Failure;
use flate2::read::GzDecoder;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

/// Where the releases are, when `$CELIS_BASE` is not set.
const BASE: &str = "https://github.com/celishq/trails/releases";

impl Command {
    /// Installs the release the version in `args` names, else the latest when it is not installed; with
    /// `--dry-run`, prints the versions and changes nothing.
    pub(super) fn update(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        if args.words.len() > 2 {
            return Err(Failure::usage("update takes one version"));
        }
        let base = self.env.get("CELIS_BASE").map_or(BASE, String::as_str);
        let base = base.trim_end_matches('/');
        let path = std::env::current_exe()
            .and_then(|p| p.canonicalize())
            .map_err(anyhow::Error::from)?;
        let from = crate::VERSION;
        let named = args.words.get(1).map(|v| v.trim_start_matches('v'));
        let to = match named {
            Some(version) => version.to_string(),
            None => latest(base)?,
        };
        let current = named.is_none() && to == from;
        let updated = !current && !args.dry_run;
        if updated {
            install(base, &to, &path)?;
        }
        let line = match (current, args.dry_run) {
            (true, _) => format!("celis {to} is the latest."),
            (false, true) => format!("celis {from} is installed; celis update installs {to}."),
            (false, false) => format!("Updated celis {from} to {to} at {}.", path.display()),
        };
        let shown = json!({"from": from, "to": to, "path": path, "updated": updated});
        match args.json {
            true => super::put(out, &shown.to_string()),
            false => super::put(out, &line),
        }
    }
}

/// The version of the release `<base>/latest` redirects to.
fn latest(base: &str) -> Result<String, Failure> {
    let url = format!("{base}/latest");
    let response = ureq::AgentBuilder::new()
        .redirects(0)
        .build()
        .get(&url)
        .call()
        .map_err(|e| anyhow!("could not reach {url}: {e}"))?;
    let version = response
        .header("location")
        .and_then(|l| l.rsplit_once("/tag/v"))
        .map(|(_, v)| v.to_string());
    Ok(version.ok_or_else(|| anyhow!("{url} names no release"))?)
}

/// Downloads the archive of `version` for this system, checks it, and puts its `celis` at `path`
/// with one rename; on a failure `path` is left as it was.
fn install(base: &str, version: &str, path: &Path) -> Result<(), Failure> {
    let url = format!("{base}/download/v{version}/{}", archive()?);
    let body = download(&url)?;
    let sum = download(&format!("{url}.sha256"))?;
    let want = String::from_utf8_lossy(&sum);
    if want.split_whitespace().next() != Some(&format!("{:x}", Sha256::digest(&body))) {
        return Err(anyhow!("the download does not match its checksum; nothing changed").into());
    }
    let new = path.with_file_name(format!(".celis-{}", std::process::id()));
    let placed = unpack(&body, &new).and_then(|()| runs(&new));
    let placed = placed.and_then(|()| Ok(std::fs::rename(&new, path)?));
    if placed.is_err() {
        let _ = std::fs::remove_file(&new);
    }
    Ok(placed?)
}

/// The archive name of this system and processor: `celis-<cpu>-<system>.tar.gz`.
fn archive() -> Result<String, Failure> {
    let system = match std::env::consts::OS {
        "macos" => "apple-darwin",
        "linux" => "unknown-linux-musl",
        os => return Err(anyhow!("there is no build for {os}").into()),
    };
    let cpu = match std::env::consts::ARCH {
        cpu @ ("aarch64" | "x86_64") => cpu,
        cpu => return Err(anyhow!("there is no build for the {cpu} processor").into()),
    };
    Ok(format!("celis-{cpu}-{system}.tar.gz"))
}

/// The body at `url`, after its redirects.
fn download(url: &str) -> Result<Vec<u8>, Failure> {
    let response = ureq::get(url)
        .call()
        .map_err(|e| anyhow!("could not download {url}: {e}"))?;
    let mut body = vec![];
    response
        .into_reader()
        .read_to_end(&mut body)
        .map_err(|e| anyhow!("could not download {url}: {e}"))?;
    Ok(body)
}

/// Writes the `celis` that the `.tar.gz` in `body` holds to `to`, runnable.
fn unpack(body: &[u8], to: &Path) -> anyhow::Result<()> {
    let mut archive = tar::Archive::new(GzDecoder::new(body));
    for entry in archive.entries()? {
        let mut entry = entry?;
        if entry.path()?.as_os_str() == "celis" {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o755)
                .open(to)
                .map_err(|e| anyhow!("could not write {}: {e}", to.display()))?;
            std::io::copy(&mut entry, &mut file)?;
            return Ok(());
        }
    }
    Err(anyhow!("the archive holds no celis; nothing changed"))
}

/// Whether the `celis` at `path` prints its version.
fn runs(path: &Path) -> anyhow::Result<()> {
    let output = std::process::Command::new(path).arg("--version").output()?;
    match output.status.success() && output.stdout.starts_with(b"celis ") {
        true => Ok(()),
        false => Err(anyhow!("the new celis does not run; nothing changed")),
    }
}
