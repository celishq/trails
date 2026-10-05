//! Keeps the commit `celis` is built from, for `celis --version`.

use std::path::Path;
use std::process::Command;

/// Trimmed stdout of `git args`, or `None` when git fails or prints nothing.
fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    let text = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (out.status.success() && !text.is_empty()).then_some(text)
}

fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let rerun = |path: String| {
        println!(
            "cargo:rerun-if-changed={}",
            Path::new(&dir).join(path).display()
        );
    };
    git(&["rev-parse", "--git-path", "HEAD"]).map(rerun);
    let head_ref = git(&["rev-parse", "--symbolic-full-name", "HEAD"]).filter(|r| r != "HEAD");
    head_ref
        .and_then(|r| git(&["rev-parse", "--git-path", &r]))
        .map(rerun);
    if let Some(commit) = git(&["rev-parse", "HEAD"]) {
        println!("cargo:rustc-env=CELIS_COMMIT={commit}");
    }
}
