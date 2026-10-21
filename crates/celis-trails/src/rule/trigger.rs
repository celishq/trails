//! What calls a rule acts on.

use celis_nostr::Note;

/// What calls a rule acts on: the files by `paths` and `content`, the shell commands by `command`, and
/// the other tool calls by `tool` (CIP-05, "Triggers").
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Trigger {
    pub paths: Vec<String>,
    pub content: Option<String>,
    pub command: Option<String>,
    pub tool: Option<String>,
}

impl Trigger {
    /// Whether it has no tag.
    pub fn is_empty(&self) -> bool {
        *self == Trigger::default()
    }

    /// Its tags: a `path` tag for each path, then `content`, `command` and `tool` when set.
    pub fn tags(&self) -> Vec<Vec<String>> {
        let paths = self.paths.iter().map(|p| ("path", p));
        let one = [
            ("content", &self.content),
            ("command", &self.command),
            ("tool", &self.tool),
        ];
        let one = one
            .into_iter()
            .filter_map(|(name, value)| Some((name, value.as_ref()?)));
        let tags = paths.chain(one);
        tags.map(|(name, value)| vec![name.to_string(), value.clone()])
            .collect()
    }

    /// The trigger the tags of `note` hold.
    pub fn read(note: &Note) -> Trigger {
        let owned = |name: &str| note.tag(name).map(str::to_string);
        Trigger {
            paths: note
                .tags
                .iter()
                .filter(|t| t.len() >= 2 && t[0] == "path")
                .map(|t| t[1].clone())
                .collect(),
            content: owned("content"),
            command: owned("command"),
            tool: owned("tool"),
        }
    }
}
