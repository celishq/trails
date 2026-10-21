//! A tool hook's JSON, as `celis rules` reads it.

use super::{changed, command, read};
use crate::failure::Failure;
use celis_trails::{Call, Place, Project};
use serde_json::Value;
use std::io::Read;
use std::path::{Path, PathBuf};

/// A tool hook: its session and folder, when it names them, and its event, tool and input, as Claude
/// Code's PreToolUse and PostToolUse give them.
pub(super) struct Hook {
    pub session: Option<String>,
    pub folder: Option<PathBuf>,
    event: String,
    tool: String,
    input: Value,
}

impl Hook {
    /// The hook on stdin; a usage error when it is no JSON.
    pub(super) fn read() -> Result<Hook, Failure> {
        let mut input = String::new();
        std::io::stdin()
            .read_to_string(&mut input)
            .map_err(anyhow::Error::from)?;
        let json: Value = serde_json::from_str(&input)
            .map_err(|_| Failure::usage("expected a hook's JSON on stdin"))?;
        let text = |key: &str| {
            json[key]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        };
        Ok(Hook {
            session: text("session_id"),
            folder: text("cwd").map(PathBuf::from),
            event: text("hook_event_name").unwrap_or_default(),
            tool: text("tool_name").unwrap_or_default(),
            input: json["tool_input"].clone(),
        })
    }

    /// The calls the hook is about, in `project`, run in `folder`: a file call, a shell command, the
    /// files a shell command changed, or another tool call.
    pub(super) fn calls(&self, project: &Project, folder: &Path) -> Vec<Call> {
        let input = &self.input;
        match self.tool.as_str() {
            "" => vec![],
            "Bash" if self.event == "PostToolUse" => changed(project),
            "Bash" => input["command"]
                .as_str()
                .map_or(vec![], |l| vec![command(project, folder, l)]),
            "Read" | "Write" | "Edit" | "MultiEdit" | "NotebookEdit" => {
                let file = input["file_path"]
                    .as_str()
                    .or(input["notebook_path"].as_str());
                let Some(file) = file else {
                    return vec![];
                };
                let Some(text) = self.text() else {
                    return vec![read(project, folder, file)];
                };
                let path = Place::resolve(file, &project.folder, folder);
                vec![Call::File {
                    path,
                    text: Some(text),
                }]
            }
            name => vec![Call::Tool {
                name: name.to_string(),
            }],
        }
    }

    /// The new text of a write or an edit: `content`, `new_string`, each `edits[].new_string`, or
    /// `new_source`.
    fn text(&self) -> Option<String> {
        let input = &self.input;
        let one = ["content", "new_string", "new_source"];
        if let Some(text) = one.iter().find_map(|k| input[k].as_str()) {
            return Some(text.to_string());
        }
        let edits = input["edits"].as_array()?;
        let edits = edits.iter().filter_map(|e| e["new_string"].as_str());
        Some(edits.collect::<Vec<_>>().join("\n"))
    }
}
