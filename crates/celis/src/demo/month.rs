//! The month `stats --demo` shows: four weeks of steps in two projects, each day's steps ending as the
//! board's sample has them, reuse growing week by week, and two trails at work now.

use super::{anchor, Demo, DAY, TASKS};
use crate::failure::Failure;
use crate::text::clock::midnight;
use celis_patterns::Scope;
use celis_store::Anchor;
use celis_trails::{Draft, TrailStore};
use std::collections::VecDeque;

impl Demo {
    /// Writes the month: four patterns a day before it, a fifth halfway; then each day's steps, a trail a
    /// project, from the morning, or today so that they close before now; then two trails at work.
    pub(super) fn month(&self) -> Result<(), Failure> {
        let (shop, crm) = (anchor("shop"), anchor("crm"));
        let first = midnight(self.now) - 27 * DAY;
        let ago = |at: u64| self.now.saturating_sub(at);
        let mut patterns = vec![];
        for (anchor, text) in PATTERNS {
            let anchor = if anchor == "shop" { &shop } else { &crm };
            patterns.push(self.write_pattern(ago(first - DAY), anchor, Scope::Project, text)?);
        }
        self.write_pattern(ago(first + 16 * DAY), &crm, Scope::Project, LATE)?;
        let mut reworked: VecDeque<String> = VecDeque::new();
        let mut applied = 0;
        for day in 0..28 {
            let trails: Vec<(&Anchor, Vec<&Step>)> = [&shop, &crm]
                .into_iter()
                .map(|a| {
                    (
                        a,
                        STEPS
                            .iter()
                            .filter(|s| s.0 == day && s.1 == a.name)
                            .collect(),
                    )
                })
                .filter(|(_, steps): &(&Anchor, Vec<&Step>)| !steps.is_empty())
                .collect();
            let span = |steps: &[&Step]| steps.iter().map(|s| s.3 + 120).sum::<u64>() + 60;
            let all: u64 = trails.iter().map(|(_, s)| span(s)).sum();
            let mut at = match day {
                27 => self
                    .now
                    .saturating_sub(all + 1_200)
                    .max(midnight(self.now) + 60),
                d => first + d as u64 * DAY + 9 * 3_600,
            };
            for (anchor, steps) in trails {
                let task = TASKS[(day + usize::from(anchor.name == "crm")) % TASKS.len()];
                let title = format!("{} · {}", task[0], crate::text::clock::day(at));
                let trail = self.trail(ago(at), anchor, &title)?;
                for (n, &&(_, _, kind, took, pattern)) in steps.iter().enumerate() {
                    at += 30;
                    let mut uses = vec![];
                    if kind == 'r' || pattern {
                        let own = if anchor.name == "shop" {
                            applied % 2
                        } else {
                            2
                        };
                        applied += usize::from(pattern);
                        uses.push(patterns[own].clone());
                    }
                    let replaces = match kind {
                        'f' => reworked.pop_front(),
                        _ => None,
                    };
                    let title = match kind {
                        'e' => format!("Ask for a review of {}", task[0].to_lowercase()),
                        _ => task[1 + n % 2].to_string(),
                    };
                    let id = self.write_step(
                        ago(at),
                        anchor,
                        Some(&trail),
                        &title,
                        &[],
                        replaces.as_deref(),
                        &uses,
                    )?;
                    at += 30;
                    match kind {
                        'e' => {
                            self.trails(ago(at))?.start(&id)?;
                        }
                        'd' => {
                            self.trails(ago(at))?.start(&id)?;
                            self.trails(ago(at + took))?.drop(&id, "not needed")?;
                        }
                        _ => self.finish(ago(at + took), took, &id, "it works", None)?,
                    }
                    at += took + 60;
                    if kind == 'w' {
                        reworked.push_back(id.clone());
                    }
                }
                at += 60;
            }
        }
        for (anchor, task, since) in [(&shop, TASKS[2], 600), (&crm, TASKS[5], 420)] {
            let trail = self.trail(since, anchor, task[0])?;
            let step = self.add(since - 30, anchor, Some(&trail), task[1], &[])?;
            self.trails(since - 240)?.start(&step)?;
        }
        Ok(())
    }

    /// Writes the new trail `title` of `anchor`, `ago` seconds before now, joining none open; its id.
    fn trail(&self, ago: u64, anchor: &Anchor, title: &str) -> Result<String, Failure> {
        let draft = Draft {
            content: title.to_string(),
            anchor: anchor.clone(),
            places: vec![],
            check: None,
            under: None,
            replaces: None,
            uses: vec![],
            misled: Vec::new(),
            session: None,
        };
        Ok(self.trails(ago)?.add(draft)?.records[0].id.clone())
    }
}

/// The patterns of the month, in force from a day before it: two of `shop`, one of `crm`.
const PATTERNS: [(&str, &str); 3] = [
    ("shop", "When you change a price: change the shelf label in the same step, because a shopper pays what the shelf says."),
    ("shop", "When you test the till: run it with an empty cart first, because an empty cart broke the total twice."),
    ("crm", "When you import contacts: match them by email before by name, because names repeat."),
];

/// The pattern of `crm` learned halfway through the month, not applied yet.
const LATE: &str =
    "When you change a contact field: keep the old value in its history, because sales asks for it.";

/// A step of the month: its day, the oldest 0; its project; how it ends, `r` done with a pattern,
/// `n` done new, `f` done new revising the oldest reworked step not yet revised, `w` done and revised
/// later, `d` dropped, `e` left open to expire; the seconds it takes; whether it applies a pattern.
type Step = (usize, &'static str, char, u64, bool);

/// Every step of the month, oldest first.
const STEPS: [Step; 90] = [
    (0, "shop", 'n', 400, false),
    (0, "crm", 'n', 950, false),
    (0, "shop", 'n', 80, false),
    (0, "crm", 'w', 300, false),
    (1, "crm", 'f', 40, false),
    (1, "shop", 'n', 360, false),
    (1, "crm", 'n', 520, false),
    (1, "shop", 'n', 380, false),
    (2, "shop", 'n', 480, false),
    (2, "crm", 'n', 70, false),
    (2, "shop", 'w', 60, false),
    (3, "crm", 'r', 50, false),
    (3, "shop", 'f', 600, false),
    (3, "crm", 'n', 2600, false),
    (3, "shop", 'n', 40, false),
    (4, "shop", 'n', 240, false),
    (4, "crm", 'n', 450, false),
    (4, "shop", 'w', 260, false),
    (4, "crm", 'd', 300, false),
    (5, "crm", 'f', 270, false),
    (6, "shop", 'n', 200, true),
    (6, "crm", 'n', 60, false),
    (7, "crm", 'r', 320, false),
    (7, "shop", 'n', 1100, false),
    (7, "crm", 'n', 160, false),
    (7, "shop", 'n', 340, false),
    (8, "shop", 'r', 180, false),
    (8, "crm", 'n', 900, false),
    (8, "shop", 'n', 140, false),
    (8, "crm", 'n', 150, false),
    (8, "shop", 'w', 100, false),
    (9, "crm", 'f', 120, true),
    (9, "shop", 'n', 1000, false),
    (9, "crm", 'd', 480, false),
    (10, "shop", 'r', 420, true),
    (10, "crm", 'n', 110, false),
    (10, "shop", 'n', 50, false),
    (10, "crm", 'e', 0, false),
    (11, "crm", 'r', 250, true),
    (11, "shop", 'n', 910, false),
    (11, "crm", 'n', 90, false),
    (11, "shop", 'w', 1000, false),
    (12, "shop", 'r', 1000, false),
    (12, "crm", 'f', 230, false),
    (13, "crm", 'r', 560, true),
    (13, "shop", 'n', 280, false),
    (14, "shop", 'r', 36, false),
    (14, "crm", 'n', 229, false),
    (14, "crm", 'n', 35, false),
    (14, "crm", 'n', 3700, false),
    (15, "shop", 'r', 55, true),
    (15, "shop", 'n', 40, false),
    (15, "shop", 'w', 960, false),
    (16, "shop", 'r', 35, true),
    (16, "shop", 'r', 30, false),
    (16, "shop", 'f', 25, false),
    (16, "crm", 'n', 40, false),
    (17, "crm", 'r', 340, true),
    (17, "shop", 'n', 450, false),
    (17, "shop", 'n', 900, false),
    (17, "shop", 'n', 260, false),
    (18, "shop", 'r', 40, true),
    (18, "crm", 'n', 32, false),
    (18, "shop", 'd', 540, false),
    (19, "crm", 'n', 400, false),
    (20, "crm", 'r', 540, false),
    (21, "shop", 'r', 229, true),
    (21, "crm", 'r', 30, false),
    (21, "crm", 'n', 33, false),
    (21, "shop", 'n', 950, false),
    (22, "crm", 'r', 30, true),
    (22, "crm", 'r', 25, false),
    (22, "shop", 'n', 1100, false),
    (22, "shop", 'n', 320, false),
    (23, "crm", 'r', 380, true),
    (23, "shop", 'r', 420, true),
    (23, "shop", 'n', 34, false),
    (23, "shop", 'n', 240, false),
    (24, "shop", 'r', 500, true),
    (24, "shop", 'r', 20, false),
    (24, "crm", 'w', 360, false),
    (25, "crm", 'r', 32, true),
    (25, "crm", 'r', 34, false),
    (25, "shop", 'f', 480, false),
    (25, "shop", 'n', 1000, false),
    (26, "crm", 'r', 1050, false),
    (26, "crm", 'e', 0, false),
    (27, "shop", 'r', 36, true),
    (27, "crm", 'r', 300, false),
    (27, "crm", 'n', 30, false),
];
