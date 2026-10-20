//! What celis says of a trail in a watch: each value it brought the trail, and how the trail stands.

use super::Tree;
use crate::terminal::remark::{Kind, Remark};
use crate::text::clock::took;
use crate::text::line::plural;
use celis_trails::{State, Step, Used};

impl Tree<'_> {
    /// Each value celis brought `trail` while it was open: a rule learned, a pitfall kept, the rules
    /// it handed a step as the step started, a rule a step kept, past work a step reused, and the trail
    /// done.
    pub(super) fn said(&self, trail: Step) -> Vec<Remark> {
        let mut said: Vec<Remark> = vec![];
        let tree = trail.tree();
        let start = trail.note().created_at;
        let end = trail.closed_at().unwrap_or(u64::MAX);
        let mut rules = self.rules(trail);
        rules.retain(|(note, _)| (start..=end).contains(&note.created_at));
        for (note, rule) in &rules {
            let pitfall = tree
                .iter()
                .filter_map(|s| s.done())
                .any(|done| done.created_at == note.created_at && done.pubkey == note.pubkey);
            let brief = rule.brief().trim_end_matches('.');
            let (what, rest) = brief.split_once(": ").unwrap_or(("", brief));
            let remark = match pitfall {
                true => Remark::new(
                    Kind::Pitfall,
                    &format!(
                        "That cost time, so **next time it won't**: {}.",
                        lower(brief)
                    ),
                    note.created_at,
                ),
                false => Remark::new(
                    Kind::Learned,
                    &format!("{what}: **{rest}**."),
                    note.created_at,
                ),
            };
            said.push(remark);
        }
        let used = self.reuse.used(self.rules);
        for step in &tree {
            let at = step.note().created_at;
            for use_ in step.uses() {
                match use_ {
                    Used::Rule(id) => {
                        let Some((_, rule)) = self.rules.get(id) else {
                            continue;
                        };
                        let brief = rule.brief().trim_end_matches('.');
                        let count = used.iter().find(|(n, ..)| n.id == id).map_or(0, |u| u.2);
                        let more = match count {
                            0 | 1 => String::new(),
                            n => format!(" **{}** guided by it so far.", plural(n, "step")),
                        };
                        let text = format!("Your agent **kept a rule**: {}.{more}", lower(brief));
                        said.push(Remark::new(Kind::Kept, &text, at));
                    }
                    Used::Follow(source) if source.trail().id() != trail.id() => {
                        let text = format!(
                            "Your agent took how “{}” was done **in {}**, **instead of starting over**.",
                            source.title(),
                            source.record().anchor.name
                        );
                        said.push(Remark::new(Kind::Reused, &text, at));
                    }
                    Used::Follow(_) => {}
                }
            }
        }
        for step in tree.iter().skip(1) {
            if let Some(remark) = self.handed(*step) {
                said.push(remark);
            }
        }
        if let (State::Done, Some(at)) = (trail.state(), trail.closed_at()) {
            let steps = tree.len().saturating_sub(1);
            let time = self
                .worked(&tree)
                .map_or(String::new(), |t| format!(" in **{}**", took(t)));
            let text = format!("**{}** done{time}.", plural(steps, "step"));
            said.push(Remark::new(Kind::Done, &text, at));
        }
        said
    }

    /// The rules celis handed `step` to keep as it started, the newest named: none when it has not
    /// started or no rule was live then.
    fn handed(&self, step: Step) -> Option<Remark> {
        let start = step.start()?;
        let mut kept = self.rules.kept(step);
        kept.retain(|(note, _)| note.created_at <= start.created_at);
        let (_, rule) = kept.iter().max_by_key(|(note, _)| note.created_at)?;
        let brief = lower(rule.brief().trim_end_matches('.'));
        let text = format!(
            "Starting “{}”, your agent got **{}** to keep, like: {brief}.",
            step.title(),
            plural(kept.len(), "rule")
        );
        Some(Remark::new(Kind::Kept, &text, start.created_at))
    }

    /// How `trail` stands: how many rules it learned and, when at least `SHOWN` did, how many of its steps
    /// started with rules in hand; or how many of its steps are done.
    pub(super) fn standing(&self, trail: Step) -> Remark {
        let tree = trail.tree();
        let start = trail.note().created_at;
        let end = trail.closed_at().unwrap_or(u64::MAX);
        let mut rules = self.rules(trail);
        rules.retain(|(note, _)| (start..=end).contains(&note.created_at));
        let steps = &tree[1..];
        let handed = steps.iter().filter(|s| self.handed(**s).is_some()).count();
        let done = steps.iter().filter(|s| s.state() == State::Done).count();
        let text = match (rules.len(), handed) {
            (0, _) => format!("**{done} of {}** done.", plural(steps.len(), "step")),
            (n, k) if k < SHOWN => format!("I learned **{}** here.", plural(n, "rule")),
            (n, k) => format!(
                "I learned **{}** here, and **{}** started with them in hand.",
                plural(n, "rule"),
                plural(k, "step")
            ),
        };
        Remark::new(Kind::Standing, &text, 0)
    }
}

/// The fewest steps with rules in hand the card names: fewer would read as celis doing nothing.
const SHOWN: usize = 3;

/// `text` with its first letter lower case.
fn lower(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map_or(String::new(), |c| c.to_lowercase().chain(chars).collect())
}
