//! What an ask taught, for the agent to keep as patterns.

use celis_nostr::Note;
use celis_patterns::Patterns;
use celis_store::records::id::short;
use celis_store::StepId;
use celis_trails::{Mark, Notes, Step};
use serde_json::json;
use std::collections::HashSet;

/// What an ask taught: what it expected, each `changed` of its steps and of the ask, each note of its
/// trail, each prompt on its trail since the ask began, the prompt that began it first, and each live
/// pattern in play, as its id and When.
pub(crate) struct Learn {
    pub ask: StepId,
    pub expect: Vec<String>,
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
        let expect = step
            .record()
            .detail()
            .filter(|l| EXPECT.iter().any(|e| l.starts_with(e)));
        Learn {
            ask: step.step_id(),
            expect: expect.map(str::to_string).collect(),
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

    /// The parts of the brief as one object (`spec/02-cli.md`, "Extract").
    pub fn json(&self) -> serde_json::Value {
        let changed: Vec<_> = self
            .changed
            .iter()
            .map(|(id, text)| json!({"step": id.to_string(), "text": text}))
            .collect();
        let patterns: Vec<_> = self
            .patterns
            .iter()
            .map(|(id, when)| json!({"id": id, "when": when}))
            .collect();
        json!({"ask": self.ask.to_string(), "expect": self.expect, "changed": changed, "notes": self.notes, "asked": self.asked, "patterns": patterns})
    }

    /// The learn brief: what the agent writes, then each line the ask taught.
    pub fn lines(&self) -> Vec<String> {
        let ask = short(&self.ask);
        let mut lines = vec![format!(
            "celis: before you answer, learn from {ask}. Write a pattern with celis pattern for each \
             choice that will come back in other work; a wish or a correction that says always, never \
             or from now on is one too. The lines below are its evidence, an after-action review: what \
             the ask expected, and what changed. A pattern never records what this ask did, such as its \
             function, its file or its fix. Before you write one, answer four questions from the \
             evidence. The cue: what in the situation showed that a choice was due? It is the When, one \
             context and its moment, never a topic, the person or the project; it starts with \"When \", \
             with no colon, and --scope says where it holds. The tension: which two goods pulled \
             against each other, and what does each side protect? They are the Forces; a defect, a \
             cost, a risk or a preference is not a good. The novice move: \
             what would someone without this lesson do? When a step, a file or a URL shows it, give it \
             as --bad. The mechanism: why does the choice resolve the tension? It is the Because; \
             \"the person asked\" is not a why. With no cue, write no pattern. With no tension, as in a \
             wish that holds always, give the person's choice as its Because: celis marks it fact, and \
             tidy moves it into the project's files. Write it in the five lines the card shows. Its \
             first line is required: Name: <the resolved form>, as a noun phrase. Its Do is one \
             choice; write each other choice as a pattern of its own, --within the larger one. When the evidence does not give \
             the mechanism, write your best guess, and name it on the Learned line for the person to \
             confirm. A reaction to a result, such as \"this one\" or \"too busy\", is a pattern of the \
             person's taste: --good the result they liked and --bad the one they did not, each a step \
             id, a file or a URL. Give --path for the module or the kind of file where it holds, as a \
             glob such as src/billing/** or **/*.svg, never one file; --within for the larger pattern \
             it completes, --scope me when it holds in any project, and --replaces <id> for a pattern \
             that misled the work, such as one below, or a near copy. When a pattern went further \
             than its evidence, as when its When also covers a case where its Do was wrong, replace \
             it with a narrower When, and write that case as a pattern of its own, never as one \
             beside it whose When names the other's rule. When a --bad is a step, say in the Forces \
             what was wrong with it: its proof says only what was built. Write nothing when the work \
             taught nothing. What holds only for this trail is a note: celis step note {ask}."
        )];
        lines.extend(self.expect.iter().map(|line| format!("  expect  {line}")));
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
/// Name.
fn in_play(records: &[Note], step: Step) -> Vec<(String, String)> {
    let trail = step.trail();
    let patterns = Patterns::new(records);
    let used = trail.tree().into_iter().flat_map(|s| s.uses());
    let mut seen = HashSet::new();
    used.filter(|id| seen.insert(*id))
        .filter_map(|id| patterns.find(id).ok())
        .map(|p| (p.id().to_string(), p.record.title().to_string()))
        .collect()
}

/// The starts of the lines of an ask's text that say what it expected.
const EXPECT: [&str; 3] = ["expect:", "for:", "not:"];
