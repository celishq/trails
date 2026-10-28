//! A sign of a weak pattern text.

use crate::pattern_record::{name, when};
use regex::Regex;
use std::sync::LazyLock;

/// A sign that a pattern's text is weak, as `patterns --tidy` lists it (CIP-02, "Patterns").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flaw {
    Form,
    Topic,
    Tension,
    Fact,
    Many,
}

impl Flaw {
    /// Each sign of a weak text that `content` shows, in the order of CIP-02, "Patterns".
    pub fn of(content: &str) -> Vec<Flaw> {
        let checks = [
            (Flaw::Form, malformed(content)),
            (Flaw::Topic, TOPIC.is_match(when(content))),
            (
                Flaw::Tension,
                field(content, "Forces:").is_some_and(one_sided),
            ),
            (Flaw::Fact, field(content, "Because:").is_some_and(choice)),
            (
                Flaw::Many,
                field(content, "Do:").is_some_and(|d| MANY.is_match(d)),
            ),
        ];
        checks
            .into_iter()
            .filter(|(_, f)| *f)
            .map(|(f, _)| f)
            .collect()
    }

    /// Its reason word.
    pub fn word(self) -> &'static str {
        match self {
            Flaw::Form => "form",
            Flaw::Topic => "topic",
            Flaw::Tension => "tension",
            Flaw::Fact => "fact",
            Flaw::Many => "many",
        }
    }

    /// What it says of the text, for the agent that wrote it.
    pub fn meaning(self) -> &'static str {
        match self {
            Flaw::Form => "the When holds another line, or starts with When:",
            Flaw::Topic => "the When names a topic, not the moment a choice is due",
            Flaw::Tension => "the Forces name no two goods that pull against each other",
            Flaw::Fact => {
                "the Because gives only the person's choice, not why it resolves the forces"
            }
            Flaw::Many => "the Do holds more than one choice",
        }
    }
}

/// The text after `label` on the first line of `content` that starts with it.
fn field<'a>(content: &'a str, label: &str) -> Option<&'a str> {
    content
        .lines()
        .find_map(|l| l.strip_prefix(label))
        .map(str::trim)
}

/// Whether the When line of `content` starts with `When:`, or holds another line of a pattern.
fn malformed(content: &str) -> bool {
    let named = name(content).is_some();
    let line = content.lines().nth(usize::from(named)).unwrap_or("");
    line.starts_with("When:")
        || ["Forces:", "Do:", "Because:"]
            .iter()
            .any(|l| line.contains(l))
}

/// Whether `forces` holds no word or mark of two sides.
fn one_sided(forces: &str) -> bool {
    !forces.contains(';') && !words(forces).any(|w| SIDES.contains(&w.as_str()))
}

/// Whether `because` gives only the person's choice.
fn choice(because: &str) -> bool {
    let lower = because.to_lowercase();
    let person = lower.starts_with("the person ") || lower.starts_with("the user ");
    let said = words(because).any(|w| CHOICES.contains(&w.as_str()));
    let why = words(because).any(|w| WHYS.contains(&w.as_str()));
    person && said && !why && !because.contains(':')
}

/// The words of `text`, in lower case.
fn words(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
}

/// A When that names a topic: something that is planned, designed and the like.
static TOPIC: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(is|are) (\w+(,| or| and) )*(planned|designed|described|discussed|explained|named|pitched|positioned|chosen)\b").unwrap()
});

/// A Do of more than one choice: a `;`, or a second sentence.
static MANY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r";|[.!?] [A-Z]").unwrap());

/// The words of Forces that name two sides.
const SIDES: [&str; 9] = [
    "but", "yet", "while", "though", "although", "whereas", "against", "versus", "pulls",
];

/// The words of a Because that give the person's choice.
const CHOICES: [&str; 20] = [
    "chose",
    "chooses",
    "wants",
    "wanted",
    "asked",
    "asks",
    "said",
    "says",
    "set",
    "stated",
    "states",
    "holds",
    "prefers",
    "preferred",
    "likes",
    "liked",
    "picked",
    "decided",
    "decides",
    "treats",
];

/// The words of a Because that give a reason beyond the choice.
const WHYS: [&str; 3] = ["because", "so", "since"];
