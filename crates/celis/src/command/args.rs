//! The words of one command line.

use crate::failure::Failure;

/// The words of one command line, split into the verb and its words, the flags and the text after `--`.
#[derive(Clone, Default)]
pub(crate) struct Args {
    pub json: bool,
    pub help: bool,
    pub graph: bool,
    pub full: bool,
    pub all: bool,
    pub demo: bool,
    pub new: bool,
    pub planned: bool,
    pub promoted: bool,
    pub next: bool,
    pub off: bool,
    pub markdown: bool,
    pub words: Vec<String>,
    pub text: Option<String>,
    pub places: Vec<String>,
    pub under: Option<String>,
    pub scope: Option<String>,
    pub replaces: Option<String>,
    pub uses: Vec<String>,
    pub check: Option<String>,
    pub changed: Option<String>,
    pub by: Option<String>,
    pub session: Option<String>,
    pub project: Option<String>,
    pub limit: Option<String>,
    pub kind: Option<String>,
    pub model: Option<String>,
    pub page: Option<String>,
    pub paths: Vec<String>,
    pub files: Vec<String>,
    pub good: Vec<String>,
    pub bad: Vec<String>,
}

impl Args {
    /// Whether `--json` is among `args` before any `--`, when they are not raw (`help::raw`).
    pub fn asks_json(args: &[String]) -> bool {
        !crate::help::raw(args)
            && args
                .iter()
                .take_while(|a| *a != "--")
                .any(|a| a == "--json")
    }

    /// The words of `args`, only the verb when they are raw; an unknown flag, or a flag without its
    /// value, is a usage error.
    pub fn parse(args: &[String]) -> Result<Args, Failure> {
        let mut parsed = Args::default();
        if crate::help::raw(args) {
            parsed.words = args[..1].to_vec();
            return Ok(parsed);
        }
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
                "--graph" => parsed.graph = true,
                "--full" => parsed.full = true,
                "--all" => parsed.all = true,
                "--demo" => parsed.demo = true,
                "--new" => parsed.new = true,
                "--as-planned" => parsed.planned = true,
                "--promoted" => parsed.promoted = true,
                "--next" => parsed.next = true,
                "--off" => parsed.off = true,
                "--markdown" => parsed.markdown = true,
                "--at" => parsed.places.push(value(arg, rest.next())?),
                "--under" => parsed.under = Some(value(arg, rest.next())?),
                "--scope" => parsed.scope = Some(value(arg, rest.next())?),
                "--replaces" => parsed.replaces = Some(value(arg, rest.next())?),
                "--uses" => parsed.uses.push(value(arg, rest.next())?),
                "--check" => parsed.check = Some(value(arg, rest.next())?),
                "--changed" => parsed.changed = Some(value(arg, rest.next())?),
                "--by" => parsed.by = Some(value(arg, rest.next())?),
                "--session" => parsed.session = Some(value(arg, rest.next())?),
                "--project" => parsed.project = Some(value(arg, rest.next())?),
                "--limit" => parsed.limit = Some(value(arg, rest.next())?),
                "--kind" => parsed.kind = Some(value(arg, rest.next())?),
                "--model" => parsed.model = Some(value(arg, rest.next())?),
                "--page" => parsed.page = Some(value(arg, rest.next())?),
                "--path" => parsed.paths.push(value(arg, rest.next())?),
                "--file" => parsed.files.push(value(arg, rest.next())?),
                "--good" => parsed.good.push(value(arg, rest.next())?),
                "--bad" => parsed.bad.push(value(arg, rest.next())?),
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

    /// The positive number `value` of `flag`; none when the flag is absent, and a usage error when it
    /// is not a positive number.
    pub fn number(flag: &str, value: &Option<String>) -> Result<Option<usize>, Failure> {
        match value.as_deref().map(|v| (v, v.parse::<usize>())) {
            None => Ok(None),
            Some((_, Ok(n))) if n > 0 => Ok(Some(n)),
            Some((v, _)) => Err(Failure::usage(format!(
                "{flag} takes a positive number, not {v}"
            ))),
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
