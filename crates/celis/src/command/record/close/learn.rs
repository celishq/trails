//! What an ask taught, for the agent to keep as patterns.

use celis_nostr::Note;
use celis_patterns::Patterns;
use celis_store::records::id::short;
use celis_store::StepId;
use celis_trails::{Mark, Notes, Step};
use std::collections::HashSet;

/// What an ask taught: each `changed` of its steps and of the ask, each note of its trail, each prompt
/// on its trail since the ask began, the prompt that began it first, and each live pattern in play, as
/// its id and When.
pub(crate) struct Learn {
    pub ask: StepId,
    pub changed: Vec<(StepId, String)>,
    pub notes: Vec<String>,
    pub asked: Vec<String>,
    pub patterns: Vec<(String, String)>,
}

impl Learn {
    /// What the ask `step`, whose own `changed` is `own`, taught, from `records`.
    pub fn of(records: &[Note], step: Step, own: Option<String>) -> Learn {
        let tree = step.tree();
        let parts = tree[1..]
            .iter()
            .filter_map(|s| Some((s.step_id(), s.done()?.changed?)));
        let own = own.map(|c| (step.step_id(), c));
        let trail = step.trail();
        let prompts: Vec<(u64, String)> = records
            .iter()
            .filter_map(|record| {
                let (on, Mark::Prompt { text }) = Mark::read(record)? else {
                    return None;
                };
                (on == trail.id()).then_some((record.created_at, text))
            })
            .collect();
        let began = step.note().created_at;
        let since = prompts
            .iter()
            .map(|(at, _)| *at)
            .filter(|at| *at <= began)
            .max()
            .unwrap_or(began);
        let asked = prompts
            .into_iter()
            .filter_map(|(at, text)| (at >= since).then_some(text));
        Learn {
            ask: step.step_id(),
            changed: parts.chain(own).collect(),
            notes: Notes::new(records)
                .of(trail)
                .iter()
                .map(|n| n.content.clone())
                .collect(),
            asked: asked.collect(),
            patterns: in_play(records, step),
        }
    }

    /// The learn brief: what the agent writes, then each line the ask taught.
    pub fn lines(&self) -> Vec<String> {
        let ask = short(&self.ask);
        let mut lines = vec![format!(
            "celis: before you answer, learn from {ask}. Write a pattern with celis pattern for each \
             choice that will come back in other work; a wish or a correction that says always, never \
             or from now on is one too. The lines below are its evidence: a pattern never records what \
             this ask did, such as its function, its file or its fix. Its When names the work, never the person or \
             the project: --scope says where it holds. Its Because says why the choice is good for the \
             work; \"the person asked\" is not a why. Take it from the person's words or the work; when \
             neither gives it, write your best guess, and name it on the Learned line for the person to \
             confirm. A reaction to a result, such as \"this one\" or \"too busy\", is a pattern of the \
             person's taste: --good the result they liked and --bad the one they did not, each a step \
             id, a file or a URL. Give --path for the files it acts on, --within for the larger pattern \
             it completes, --scope me when it holds in any project, and --replaces <id> for a pattern \
             that misled the work, such as one below, or a near copy. Write nothing when the work \
             taught nothing. What holds only for this trail is a note: celis step note {ask}."
        )];
        for (id, text) in &self.changed {
            lines.push(format!("  changed {} {text}", short(id)));
        }
        for note in &self.notes {
            let mut text = note.lines();
            lines.extend(text.next().map(|first| format!("  note    {first}")));
            lines.extend(text.map(|line| format!("          {line}")));
        }
        lines.extend(self.asked.iter().map(|text| format!("  asked   {text}")));
        for (id, when) in &self.patterns {
            lines.push(format!("  pattern {} {when}", short(id)));
        }
        lines
    }
}

/// Each live pattern of `records` that a step of the tree of the ask of `step` uses, once, as its id and
/// When.
fn in_play(records: &[Note], step: Step) -> Vec<(String, String)> {
    let trail = step.trail();
    let patterns = Patterns::new(records);
    let used = trail.tree().into_iter().flat_map(|s| s.uses());
    let mut seen = HashSet::new();
    used.filter(|id| seen.insert(*id))
        .filter_map(|id| patterns.find(None, id).ok())
        .map(|p| (p.id().to_string(), p.record.when().to_string()))
        .collect()
}
