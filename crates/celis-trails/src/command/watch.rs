//! `celis watch`: a trail's tree, printed again each time the store changes.

use super::tree::Tree;
use super::{Args, Command};
use crate::steps::ACTIVE;
use crate::{Decisions, Failure, Project, Steps, Store};
use notify::{RecursiveMode, Watcher};
use std::io::{IsTerminal, Write};
use std::sync::mpsc::channel;
use std::time::Duration;

impl Command {
    /// Prints the tree of the step `args` names, or with no id the newest active trail of `project`, then
    /// prints it again each time a record lands and the tree changed. Runs until it is stopped.
    pub(super) fn watch(
        &self,
        args: &Args,
        out: &mut dyn Write,
        project: &Project,
    ) -> Result<(), Failure> {
        let records = self.store()?.records();
        let steps = Steps { records: &records };
        let id = match args.words.get(1) {
            Some(id) => Some(steps.find(id)?.id.clone()),
            None => None,
        };
        let (send, events) = channel();
        let mut watcher = notify::recommended_watcher(send).map_err(anyhow::Error::from)?;
        watcher
            .watch(
                &self.home()?.join("trails/shared"),
                RecursiveMode::NonRecursive,
            )
            .map_err(anyhow::Error::from)?;
        let terminal = std::io::stdout().is_terminal();
        let mut shown = String::new();
        loop {
            let tree = self.frame(id.as_deref(), project, args.json)?;
            if !tree.is_empty() && tree != shown {
                let gap = match (terminal, shown.is_empty()) {
                    (true, _) => "\x1b[H\x1b[2J\x1b[3J",
                    (false, true) => "",
                    (false, false) => "\n",
                };
                write!(out, "{gap}{tree}")
                    .and_then(|_| out.flush())
                    .map_err(anyhow::Error::from)?;
                shown = tree;
            }
            if events.recv().is_err() {
                return Ok(());
            }
            while events.recv_timeout(Duration::from_millis(50)).is_ok() {}
        }
    }

    /// The tree `watch` prints now: of the step `id`, or of the newest active trail of `project`; empty
    /// with no such trail.
    fn frame(&self, id: Option<&str>, project: &Project, json: bool) -> Result<String, Failure> {
        let store = Store::open(&self.home()?)?;
        let records = store.records();
        let steps = Steps { records: &records };
        let decisions = Decisions {
            records: &records,
            account: store.account().pubkey(),
        };
        let note = match id {
            Some(id) => steps.find(id).ok(),
            None => steps
                .trails(&project.id)
                .into_iter()
                .rfind(|n| steps.state(n) == ACTIVE),
        };
        let mut printed = Vec::new();
        if let Some(note) = note {
            let tree = Tree {
                steps: &steps,
                decisions: &decisions,
            };
            tree.print(note, "trail", json, &mut printed)?;
        }
        Ok(String::from_utf8_lossy(&printed).into_owned())
    }
}
