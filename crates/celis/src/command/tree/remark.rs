//! What celis says of a trail in a watch: the newest value it brought the trail, else how it stands.

use super::Tree;
use crate::terminal::remark::{Kind, Remark};
use crate::text::clock::took;
use crate::text::line::plural;
use celis_trails::{State, Step, Used};

impl Tree<'_> {
    /// The newest of what celis brought `trail` while it was open: a rule learned, a pitfall kept, a
    /// rule a step kept, past work a step reused, or the trail done; else how many rules it learned and
    /// how often they were kept, or how many of its steps are done.
    pub(super) fn remark(&self, trail: Step) -> Remark {
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
        if let (State::Done, Some(at)) = (trail.state(), trail.closed_at()) {
            let steps = tree.len().saturating_sub(1);
            let time = self
                .worked(&tree)
                .map_or(String::new(), |t| format!(" in **{}**", took(t)));
            let text = format!("**{}** done{time}.", plural(steps, "step"));
            said.push(Remark::new(Kind::Done, &text, at));
        }
        if let Some(newest) = said.into_iter().max_by_key(|r| r.at) {
            return newest;
        }
        let kept: usize = rules
            .iter()
            .filter_map(|(note, _)| used.iter().find(|(n, ..)| n.id == note.id))
            .map(|u| u.2)
            .sum();
        let steps = &tree[1..];
        let done = steps.iter().filter(|s| s.state() == State::Done).count();
        let text = match rules.len() {
            0 => format!("**{done} of {}** done.", plural(steps.len(), "step")),
            n => format!(
                "I learned **{}** here, and your agents **kept them {}**.",
                plural(n, "rule"),
                plural(kept, "time")
            ),
        };
        Remark::new(Kind::Standing, &text, 0)
    }
}

/// `text` with its first letter lower case.
fn lower(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map_or(String::new(), |c| c.to_lowercase().chain(chars).collect())
}
