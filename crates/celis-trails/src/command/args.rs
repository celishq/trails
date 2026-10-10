//! The words of one command line.

use crate::Failure;

/// The words of one command line, split into the verb and its words, the flags and the text after `--`.
#[derive(Default)]
pub(crate) struct Args {
    pub json: bool,
    pub help: bool,
    pub words: Vec<String>,
    pub text: Option<String>,
    pub at: Option<String>,
    pub places: Vec<String>,
    pub under: Option<String>,
    pub fixes: Option<String>,
    pub follows: Option<String>,
    pub scope: Option<String>,
    pub replaces: Option<String>,
    pub files: Vec<String>,
    pub check: Option<String>,
    pub like: Option<String>,
}

impl Args {
    /// Whether `--json` is among `args` before any `--`.
    pub fn asks_json(args: &[String]) -> bool {
        args.iter()
            .take_while(|a| *a != "--")
            .any(|a| a == "--json")
    }

    /// The words of `args`; an unknown flag, or a flag without its value, is a usage error.
    pub fn parse(args: &[String]) -> Result<Args, Failure> {
        let mut parsed = Args::default();
        let mut bad = None;
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            match arg.as_str() {
                "--" => {
                    let text: Vec<&str> = rest.by_ref().map(String::as_str).collect();
                    parsed.text = Some(text.join(" "));
                }
                "--json" => parsed.json = true,
                "--help" => parsed.help = true,
                "--at" => {
                    let place = value(arg, rest.next())?;
                    parsed.places.push(place.clone());
                    parsed.at = Some(place);
                }
                "--under" => parsed.under = Some(value(arg, rest.next())?),
                "--fixes" => parsed.fixes = Some(value(arg, rest.next())?),
                "--follows" => parsed.follows = Some(value(arg, rest.next())?),
                "--scope" => parsed.scope = Some(value(arg, rest.next())?),
                "--replaces" => parsed.replaces = Some(value(arg, rest.next())?),
                "--file" => parsed.files.push(value(arg, rest.next())?),
                "--check" => parsed.check = Some(value(arg, rest.next())?),
                "--like" => parsed.like = Some(value(arg, rest.next())?),
                flag if flag.starts_with("--") => {
                    bad.get_or_insert(Failure::usage(format!("{flag} is not a flag")));
                }
                word => parsed.words.push(word.to_string()),
            }
        }
        match bad {
            Some(failure) if !parsed.help => Err(failure),
            _ => Ok(parsed),
        }
    }

    /// The text after `--`; a usage error with `missing` when it is absent or blank.
    pub fn required_text(&self, missing: &str) -> Result<String, Failure> {
        match &self.text {
            Some(text) if !text.trim().is_empty() => Ok(text.clone()),
            _ => Err(Failure::usage(missing)),
        }
    }
}

fn value(flag: &str, next: Option<&String>) -> Result<String, Failure> {
    next.cloned()
        .ok_or_else(|| Failure::usage(format!("{flag} needs a value")))
}
