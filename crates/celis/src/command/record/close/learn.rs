//! What an ask taught, for the agent to keep as patterns when the ask closes.

use celis_nostr::Note;
use celis_store::records::id::short;
use celis_store::StepId;
use celis_trails::{Done, Mark, Notes, Step};

/// What an ask taught: each `changed` of its steps and of the ask, each note of its trail, and each prompt
/// on its trail since the ask began, the prompt that began it first.
pub(crate) struct Learn {
    pub ask: StepId,
    pub changed: Vec<(StepId, String)>,
    pub notes: Vec<String>,
    pub asked: Vec<String>,
}

impl Learn {
    /// What the ask `step`, just closed `done`, taught, from `records`.
    pub fn of(records: &[Note], step: Step, done: &Done) -> Learn {
        let tree = step.tree();
        let parts = tree[1..]
            .iter()
            .filter_map(|s| Some((s.step_id(), s.done()?.changed?)));
        let own = done.changed.clone().map(|c| (step.step_id(), c));
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
        }
    }

    /// The learn brief: what the agent writes, then each line the ask taught.
    pub fn lines(&self) -> Vec<String> {
        let ask = short(&self.ask);
        let mut lines = vec![format!(
            "celis: before you answer, learn from the ask {ask}. Write a pattern with celis pattern for \
             each choice of the ask that will come back, such as those below. A correction that says \
             always or never is a pattern whose forces say why. A reaction to a result, such as \"this \
             one\" or \"too busy\", keeps the person's taste with --good and --bad. Give --path for the \
             files it acts on, --within for the larger pattern it completes, --scope me when it holds in \
             any project, and --replaces <id> for a pattern that misled you or a near copy. Write nothing \
             when the ask taught nothing. What holds only for this trail is a note: celis step note {ask}."
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
        lines
    }
}
