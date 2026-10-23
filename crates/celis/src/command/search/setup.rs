//! `celis setup`: the model that `search` uses.

use super::model::{Model, MODELS};
use crate::command::{put, Args, Command};
use crate::failure::Failure;
use anyhow::anyhow;
use sha2::{Digest, Sha256};
use std::io::{BufRead, IsTerminal, Write};
use std::path::Path;
use std::process::Stdio;

/// Where `setup` downloads models from when `CELIS_MODELS_BASE` is not set.
const BASE: &str = "https://huggingface.co";

/// The line of `none` in the list.
const NONE: &str = "none 0 MB - - turns search off; agents use celis grep";

impl Command {
    /// Sets up the model `--model` names: downloads and checks it, then indexes the records. With no
    /// model, asks for one on a terminal, and else prints the list.
    pub(crate) fn setup(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> Result<(), Failure> {
        let terminal = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
        let id = match (&args.model, terminal) {
            (Some(id), _) => id.clone(),
            (None, true) => match ask(out)? {
                Some(id) => id,
                None => return Ok(()),
            },
            (None, false) => {
                for model in MODELS {
                    put(out, &model.line())?;
                }
                return put(out, NONE);
            }
        };
        let home = self.home()?;
        if id == "none" {
            remove(&home.join("models"))?;
            remove(&home.join("index"))?;
            return put(out, "celis search is off.");
        }
        let model = Model::named(&id).ok_or_else(|| {
            Failure::usage(format!(
                "--model takes embeddinggemma, e5-small or none, not {id}"
            ))
        })?;
        self.download(model, &home)?;
        let rows = self.searched()?;
        self.indexed(model, &rows, err)?;
        put(out, &format!("celis search uses {}.", model.id))
    }

    /// Downloads the files of `model` and checks their sha256, then puts them in place of any other
    /// model; on a failure, keeps the model there was.
    fn download(&self, model: &Model, home: &Path) -> Result<(), Failure> {
        let base = self
            .env
            .get("CELIS_MODELS_BASE")
            .map_or(BASE, String::as_str);
        let part = home.join("models").join(format!(".{}.part", model.id));
        remove(&part)?;
        let shown = std::io::stderr().is_terminal();
        for (file, sha) in model.files {
            let url = format!(
                "{}/{}/resolve/{}/{file}",
                base.trim_end_matches('/'),
                model.repo,
                model.revision
            );
            let target = part.join(file);
            let status = std::process::Command::new("curl")
                .args(["-fL", "--create-dirs", "-o"])
                .arg(&target)
                .arg(if shown { "--progress-bar" } else { "-sS" })
                .arg(&url)
                .stderr(Stdio::inherit())
                .status()
                .map_err(|e| anyhow!("could not run curl: {e}"))?;
            if !status.success() {
                remove(&part)?;
                return Err(anyhow!("could not download {url}").into());
            }
            if hash(&target)? != *sha {
                remove(&part)?;
                return Err(anyhow!("{file} of {} does not match its sha256", model.id).into());
            }
        }
        remove(&home.join("index"))?;
        for other in MODELS {
            remove(&other.folder(home))?;
        }
        std::fs::rename(&part, model.folder(home)).map_err(anyhow::Error::from)?;
        Ok(())
    }
}

/// The model a person picks from the list on the terminal; none when they decline its license.
fn ask(out: &mut dyn Write) -> Result<Option<String>, Failure> {
    let choices: Vec<String> = MODELS
        .iter()
        .map(Model::line)
        .chain([NONE.into()])
        .collect();
    for (n, line) in choices.iter().enumerate() {
        put(out, &format!("  {}. {line}", n + 1))?;
    }
    let answer = read(out, "Choose a model [1]: ")?;
    let n = match answer.trim() {
        "" => 1,
        word => word.parse::<usize>().unwrap_or(0),
    };
    let Some(model) = MODELS.get(n.wrapping_sub(1)) else {
        return match n == MODELS.len() + 1 {
            true => Ok(Some("none".into())),
            false => Err(Failure::usage(format!(
                "choose a number from 1 to {}",
                choices.len()
            ))),
        };
    };
    let question = format!("Use {} under {}? [y/N]: ", model.id, model.license);
    let yes = read(out, &question)?.trim().eq_ignore_ascii_case("y");
    Ok(yes.then(|| model.id.to_string()))
}

/// The line a person types after `question`.
fn read(out: &mut dyn Write, question: &str) -> Result<String, Failure> {
    write!(out, "{question}").map_err(anyhow::Error::from)?;
    out.flush().map_err(anyhow::Error::from)?;
    let mut line = String::new();
    std::io::stdin()
        .lock()
        .read_line(&mut line)
        .map_err(anyhow::Error::from)?;
    Ok(line)
}

/// The sha256 of the file `path`, in hex.
fn hash(path: &Path) -> Result<String, Failure> {
    let mut file = std::fs::File::open(path).map_err(anyhow::Error::from)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(anyhow::Error::from)?;
    let digest = hasher.finalize();
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

/// Removes the folder `path` and all in it, when it exists.
fn remove(path: &Path) -> Result<(), Failure> {
    match std::fs::remove_dir_all(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(anyhow::Error::from(e).into()),
        _ => Ok(()),
    }
}
