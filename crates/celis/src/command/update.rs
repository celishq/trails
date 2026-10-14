//! `celis update`: the running `celis`, replaced with the build of a release by its `install.sh`.

use super::{Args, Command};
use anyhow::anyhow;
use celis_trails::Failure;
use std::io::Write;
use std::process::Stdio;

/// Where the releases are, when `$CELIS_BASE` is not set.
const BASE: &str = "https://github.com/celishq/trails/releases";

impl Command {
    /// Runs the latest release's `install.sh` into the folder of the running `celis`, for the version in
    /// `args`, else the latest when it is not installed.
    pub(super) fn update(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        if args.words.len() > 2 {
            return Err(Failure::usage("update takes one version"));
        }
        let base = self.env.get("CELIS_BASE").map_or(BASE, String::as_str);
        let base = base.trim_end_matches('/');
        let path = std::env::current_exe()
            .and_then(|p| p.canonicalize())
            .map_err(anyhow::Error::from)?;
        let folder = path
            .parent()
            .ok_or_else(|| anyhow!("celis is in no folder"))?;
        let version = args
            .words
            .get(1)
            .map_or("latest", |v| v.trim_start_matches('v'));
        let url = format!("{base}/latest/download/install.sh");
        let script = std::process::Command::new("curl")
            .args(["-fsSL", &url])
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| anyhow!("could not run curl: {e}"))?;
        if !script.status.success() {
            return Err(anyhow!("could not download {url}").into());
        }
        let mut sh = std::process::Command::new("sh")
            .env("CELIS_BASE", base)
            .env("CELIS_VERSION", version)
            .env("CELIS_INSTALL_DIR", folder)
            .env("CELIS_UPDATE", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("could not run sh: {e}"))?;
        sh.stdin
            .take()
            .ok_or_else(|| anyhow!("could not write to sh"))?
            .write_all(&script.stdout)
            .map_err(anyhow::Error::from)?;
        let shown = sh.wait_with_output().map_err(anyhow::Error::from)?;
        out.write_all(&shown.stdout).map_err(anyhow::Error::from)?;
        match shown.status.success() {
            true => Ok(()),
            false => Err(anyhow!("install.sh failed; see above").into()),
        }
    }
}
