//! The words of one command line.

use celis_trails::Failure;

/// The words of one command line, split into the verb and its words, the flags and the text after `--`.
#[derive(Default)]
pub(crate) struct Args {
    pub json: bool,
    pub help: bool,
    pub full: bool,
    pub all: bool,
    pub new: bool,
    pub planned: bool,
    pub words: Vec<String>,
    pub text: Option<String>,
    pub places: Vec<String>,
    pub under: Option<String>,
    pub scope: Option<String>,
    pub replaces: Option<String>,
    pub check: Option<String>,
    pub pitfall: Option<String>,
    pub changed: Option<String>,
    pub session: Option<String>,
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
                "--full" => parsed.full = true,
                "--all" => parsed.all = true,
                "--new" => parsed.new = true,
                "--as-planned" => parsed.planned = true,
                "--at" => parsed.places.push(value(arg, rest.next())?),
                "--under" => parsed.under = Some(value(arg, rest.next())?),
                "--scope" => parsed.scope = Some(value(arg, rest.next())?),
                "--replaces" => parsed.replaces = Some(value(arg, rest.next())?),
                "--check" => parsed.check = Some(value(arg, rest.next())?),
                "--pitfall" => parsed.pitfall = Some(value(arg, rest.next())?),
                "--changed" => parsed.changed = Some(value(arg, rest.next())?),
                "--session" => parsed.session = Some(value(arg, rest.next())?),
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

    /// The id after the words of `command`; a usage error when it is absent.
    pub fn id(&self, command: &str) -> Result<&str, Failure> {
        self.words
            .get(command.split(' ').count())
            .map(String::as_str)
            .ok_or_else(|| Failure::usage(format!("{command} needs an id")))
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
