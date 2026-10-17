//! `celis auth`: the account's phrase.

use super::{put, show, Args, Command};
use crate::failure::Failure;
use celis_nostr::Identity;
use celis_trails::Phrase;
use serde_json::json;
use std::io::{IsTerminal, Read, Write};

impl Command {
    /// Runs the `auth` command its second word names.
    pub(crate) fn auth(
        &self,
        args: &Args,
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> Result<(), Failure> {
        let phrase = Phrase::at(&self.home()?);
        match args.words.get(1).map(String::as_str).unwrap_or("") {
            "login" => login(args, out, err, &phrase),
            "status" => {
                let npub = kept(&phrase)?.account().pubkey().npub();
                let path = phrase.path().display().to_string();
                match args.json {
                    true => put(out, &json!({"npub": npub, "path": path}).to_string()),
                    false => put(out, &format!("{npub}\n{path}")),
                }
            }
            "phrase" => {
                let words = kept(&phrase)?.phrase();
                show(out, args.json, "phrase", json!(words), &words)
            }
            "" => Err(Failure::usage(
                "expected: celis auth login | status | phrase",
            )),
            verb => Err(crate::help::unknown(verb, ["login", "status", "phrase"])),
        }
    }
}

/// Keeps the phrase stdin gives in place of the key of older versions, and prints the npub of its
/// account.
fn login(
    args: &Args,
    out: &mut dyn Write,
    err: &mut dyn Write,
    phrase: &Phrase,
) -> Result<(), Failure> {
    if args.words.len() > 2 || args.text.is_some() {
        return Err(Failure::usage(
            "give the phrase on stdin, so your shell history never holds it",
        ));
    }
    let text = match std::io::stdin().is_terminal() {
        true => asked(err)?,
        false => {
            let mut text = String::new();
            std::io::stdin()
                .read_to_string(&mut text)
                .map_err(anyhow::Error::from)?;
            text
        }
    };
    let words: Vec<&str> = text.split_whitespace().collect();
    let identity = Identity::from_phrase(&words.join(" "))?;
    phrase.keep(&identity)?;
    let _ = std::fs::remove_file(phrase.path().with_file_name("key"));
    let npub = identity.account().pubkey().npub();
    show(out, args.json, "npub", json!(npub), &npub)
}

/// The identity of the kept phrase; an error that names `celis auth login` when none is kept.
fn kept(phrase: &Phrase) -> Result<Identity, Failure> {
    phrase.kept()?.ok_or_else(|| {
        Failure::from(anyhow::anyhow!(
            "no phrase is kept; give one with `celis auth login`, or write a record to mint one"
        ))
    })
}

/// One line read from the terminal after a prompt on `err`, with echo off while it is typed.
fn asked(err: &mut dyn Write) -> Result<String, Failure> {
    let _ = write!(err, "Recovery phrase: ");
    let _ = err.flush();
    let mut was = unsafe { std::mem::zeroed::<libc::termios>() };
    let held = unsafe { libc::tcgetattr(libc::STDIN_FILENO, &mut was) } == 0;
    if held {
        let mut quiet = was;
        quiet.c_lflag &= !libc::ECHO;
        unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &quiet) };
    }
    let mut line = String::new();
    let read = std::io::stdin().read_line(&mut line);
    if held {
        unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &was) };
    }
    let _ = writeln!(err);
    read.map_err(anyhow::Error::from)?;
    Ok(line)
}
