//! The demo of `tree --graph`: eleven trails of today in five projects, the biggest of over a hundred
//! steps, written before the view opens, and the rest of their events written live, one a beat.

use crate::failure::Failure;
use celis_trails::{Anchor, Draft, PatternRecord, Scope, Store, Trails};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

/// The time between two events written live.
const BEAT: Duration = Duration::from_millis(700);

/// The time before the first event written live.
const OPEN: Duration = Duration::from_millis(1600);

/// One trail of the demo: its title, project, steps one a line, nested by two spaces; its patterns, each
/// with the step it was learned after, by its line; the steps dropped and those done otherwise than
/// planned; from which top step on it plays live and how many steps of each it closes then; from which
/// top step on its steps stay open; whether it writes all its events before the view opens, whether it
/// finishes, and how many beats in it starts.
struct Plan {
    title: &'static str,
    project: &'static str,
    steps: &'static str,
    patterns: &'static [(&'static str, usize)],
    dropped: &'static [&'static str],
    changed: &'static [&'static str],
    live_from: usize,
    live_done: &'static [usize],
    open_from: Option<usize>,
    quiet: bool,
    finish: bool,
    start_after: usize,
}

/// One thing that happens on a trail: a step added, at work or open; a step done or dropped; a pattern
/// learned.
#[derive(Clone, Copy)]
enum Event {
    Add(usize, bool),
    Close(usize, bool),
    Learn(usize),
}

/// A trail as the demo writes it: its plan, each step's title and parent, its events before the view
/// opens and after, and the ids written so far.
struct Written {
    plan: &'static Plan,
    nodes: Vec<(String, Option<usize>)>,
    before: Vec<Event>,
    after: Vec<Event>,
    ids: Vec<Option<String>>,
    trail: Option<String>,
}

/// The events written live, and the thread that writes them until it is told to stop.
pub(super) struct Play {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Play {
    /// Writes in the store at `home` every event of the demo before the view opens, the last of each
    /// trail within its life before `now`; then, when `live`, starts writing the rest one a beat.
    pub fn start(home: &Path, now: u64, live: bool) -> Result<Play, Failure> {
        let mut trails: Vec<Written> = PLANS.iter().map(Written::of).collect();
        let mut store = Trails::open(Store::open(home)?, now, 600);
        for (n, trail) in trails.iter_mut().enumerate() {
            let count = trail.before.len().max(1) as u64;
            let (from, to) = (START[n], 120 + 10 * n as u64);
            let gap = from.saturating_sub(to) / count;
            for (k, event) in trail.before.clone().into_iter().enumerate() {
                trail.write(&mut store, home, event, now - from + k as u64 * gap)?;
            }
        }
        let stop = Arc::new(AtomicBool::new(false));
        if !live {
            return Ok(Play { stop, thread: None });
        }
        let queue = order(&trails);
        let home: PathBuf = home.to_path_buf();
        let halt = stop.clone();
        let thread = std::thread::spawn(move || {
            let pause = |length: Duration| {
                let end = std::time::Instant::now() + length;
                while std::time::Instant::now() < end {
                    if halt.load(Ordering::Relaxed) {
                        return false;
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
                true
            };
            if !pause(OPEN) {
                return;
            }
            let Ok(store) = Store::open(&home) else {
                return;
            };
            let mut store = Trails::open(store, crate::command::now(), 600);
            for (n, event) in queue {
                if !pause(BEAT) {
                    return;
                }
                if trails[n]
                    .write(&mut store, &home, event, crate::command::now())
                    .is_err()
                {
                    return;
                }
            }
        });
        Ok(Play {
            stop,
            thread: Some(thread),
        })
    }

    /// Stops writing, and waits for the writer to end.
    pub fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Written {
    /// The trail `plan` gives, its events split at the view's opening: a trail that plays live and
    /// starts with the view has its first step at work before, so it is at work when the view opens.
    fn of(plan: &'static Plan) -> Written {
        let nodes = nodes(plan.steps);
        let tops: Vec<usize> = (0..nodes.len()).filter(|n| nodes[*n].1.is_none()).collect();
        let kids = |of: usize| -> Vec<usize> {
            (0..nodes.len())
                .filter(|n| nodes[*n].1 == Some(of))
                .collect()
        };
        let (mut before, mut after) = (vec![], vec![]);
        let learned = |at: usize, into: &mut Vec<Event>| {
            for (r, (_, after)) in plan.patterns.iter().enumerate() {
                if *after == at {
                    into.push(Event::Learn(r));
                }
            }
        };
        for (n, top) in tops.iter().copied().enumerate() {
            let into = match n < plan.live_from || plan.quiet {
                true => &mut before,
                false => &mut after,
            };
            if plan.open_from.is_some_and(|from| n >= from) {
                into.push(Event::Add(top, false));
                continue;
            }
            into.push(Event::Add(top, true));
            learned(top, into);
            let dropped = |s: usize| plan.dropped.contains(&nodes[s].0.as_str());
            if n < plan.live_from {
                for kid in kids(top) {
                    into.push(Event::Add(kid, true));
                    for grand in kids(kid) {
                        into.push(Event::Add(grand, true));
                        into.push(Event::Close(grand, true));
                    }
                    learned(kid, into);
                    into.push(Event::Close(kid, !dropped(kid)));
                }
                into.push(Event::Close(top, !dropped(top)));
                continue;
            }
            let done = plan.live_done[n - plan.live_from];
            for (k, kid) in kids(top).into_iter().enumerate() {
                if k > done {
                    into.push(Event::Add(kid, false));
                    continue;
                }
                into.push(Event::Add(kid, true));
                let grands = kids(kid);
                for (g, grand) in grands.iter().copied().enumerate() {
                    into.push(Event::Add(grand, true));
                    if k < done || g + 1 < grands.len() {
                        into.push(Event::Close(grand, true));
                    }
                }
                if k == done {
                    continue;
                }
                learned(kid, into);
                into.push(Event::Close(kid, !dropped(kid)));
            }
            if plan.finish {
                into.push(Event::Close(top, true));
            }
        }
        if plan.start_after == 0 {
            while !after.is_empty() {
                let event = after.remove(0);
                before.push(event);
                if let Event::Add(_, true) = event {
                    break;
                }
            }
        }
        let count = nodes.len();
        Written {
            plan,
            nodes,
            before,
            after,
            ids: vec![None; count],
            trail: None,
        }
    }

    /// Writes `event` in `store` at the second `at`, the store's folder `home`, reading the store again
    /// before each record: a step under its parent or the trail, started when it is
    /// at work; a step done, with what changed when it was done otherwise than planned, or dropped; a
    /// pattern of the trail.
    fn write(
        &mut self,
        store: &mut Trails,
        home: &Path,
        event: Event,
        at: u64,
    ) -> Result<(), Failure> {
        let anchor = Anchor {
            project: format!("demo-{}", self.plan.project),
            name: self.plan.project.to_string(),
        };
        match event {
            Event::Add(node, working) => {
                let trail = match &self.trail {
                    Some(trail) => trail.clone(),
                    None => {
                        store.at(at);
                        let draft = draft(&anchor, self.plan.title, None);
                        let id = store.add(draft)?.records[0].id.clone();
                        self.trail = Some(id.clone());
                        id
                    }
                };
                let (title, parent) = &self.nodes[node];
                let under = match parent {
                    Some(p) => self.ids[*p].clone(),
                    None => Some(trail),
                };
                store.at(at);
                let id = store.add(draft(&anchor, title, under))?.records[0]
                    .id
                    .clone();
                if working {
                    store.at(at);
                    store.start(&id)?;
                }
                self.ids[node] = Some(id);
            }
            Event::Close(node, done) => {
                let Some(id) = self.ids[node].clone() else {
                    return Ok(());
                };
                let title = self.nodes[node].0.as_str();
                store.at(at);
                match done {
                    true => {
                        let changed = self
                            .plan
                            .changed
                            .contains(&title)
                            .then(|| "it took another way".to_string());
                        store.done(
                            &id,
                            celis_trails::Done::new("it works".to_string(), changed),
                            home,
                        )?;
                    }
                    false => {
                        store.drop(&id, "no longer needed")?;
                    }
                }
            }
            Event::Learn(r) => {
                let Some(trail) = self.trail.clone() else {
                    return Ok(());
                };
                let pattern = PatternRecord {
                    anchor,
                    scope: Scope::Trail(trail),
                    replaces: None,
                    content: self.plan.patterns[r].0.to_string(),
                    paths: vec![],
                    binds: false,
                    within: vec![],
                    examples: Default::default(),
                };
                store.at(at);
                store.decide(pattern, None)?;
            }
        }
        Ok(())
    }
}

/// The events written live in turn: the biggest trail every other beat and the others taking turns,
/// each not before it starts.
fn order(trails: &[Written]) -> Vec<(usize, Event)> {
    let mut queues: Vec<std::collections::VecDeque<Event>> = trails
        .iter()
        .map(|t| t.after.iter().copied().collect())
        .collect();
    let (mut beat, mut turn, mut out) = (0usize, 0usize, vec![]);
    while queues.iter().any(|q| !q.is_empty()) {
        let ready: Vec<usize> = (1..queues.len())
            .filter(|n| !queues[*n].is_empty() && beat >= trails[*n].plan.start_after)
            .collect();
        let pick = match (
            !queues[0].is_empty() && (beat % 2 == 0 || ready.is_empty()),
            ready.is_empty(),
        ) {
            (true, _) => 0,
            (false, false) => {
                turn += 1;
                ready[(turn - 1) % ready.len()]
            }
            (false, true) => {
                beat += 1;
                continue;
            }
        };
        if let Some(event) = queues[pick].pop_front() {
            out.push((pick, event));
        }
        beat += 1;
    }
    out
}

/// The draft of the step `text` of `anchor` under `under`, or a trail with none.
fn draft(anchor: &Anchor, text: &str, under: Option<String>) -> Draft {
    Draft {
        content: text.to_string(),
        anchor: anchor.clone(),
        places: vec![],
        check: None,
        under,
        replaces: None,
        uses: vec![],
        session: None,
        new: false,
    }
}

/// Each line of `steps` as its title and the line it is under, by two spaces of nesting.
fn nodes(steps: &str) -> Vec<(String, Option<usize>)> {
    let mut nodes: Vec<(String, Option<usize>)> = vec![];
    let mut path: Vec<usize> = vec![];
    for line in steps.lines().filter(|l| !l.trim().is_empty()) {
        let depth = (line.len() - line.trim_start().len()) / 2;
        path.truncate(depth);
        nodes.push((line.trim().to_string(), path.last().copied()));
        path.push(nodes.len() - 1);
    }
    nodes
}

/// The seconds before now each trail's first event is written, in the order of `PLANS`, so the newer
/// trails of a project come later.
const START: [u64; 11] = [
    3 * 3600,
    2400,
    9000,
    7200,
    6300,
    5400,
    4500,
    3600,
    3000,
    2700,
    1800,
];

/// The trails of the demo, the oldest first.
const PLANS: [Plan; 11] = [
    Plan {
        title: "Build the checkout for the shop",
        project: "shop",
        steps: CHECKOUT,
        patterns: &[
            ("When you add up money: count in cents as integers, because floats lost a cent on three items.", 2),
            ("When you price a senior: pay 80% of the price, rounded down to the cent, because the receipt must match the shelf.", 6),
            ("When you show a price: use the shop's currency, because the receipt showed dollars.", 9),
            ("When a form shows an error: put it under its field and move focus there, because a list at the top was missed.", 16),
            ("When the card is entered: keep it in the provider's frame, because the server must never see a card number.", 27),
            ("When a payment times out: ask the provider before charging again, because a retry charged a buyer twice.", 31),
            ("When stock is held: free it after 15 minutes, because abandoned carts emptied the shelf.", 39),
            ("When two coupons apply: take the larger first, because the order changed the total.", 44),
            ("When tax is rounded: round per line, not per order, because the receipt must add up line by line.", 51),
            ("When an order changes state: mail the buyer once, because two mails a change read as spam.", 63),
            ("When a refund is partial: refund the tax of those lines too, because the buyer was charged tax on returns.", 69),
            ("When a payment is held: never tell the buyer why, because a reason teaches a fraudster.", 76),
            ("When the checkout is measured: time it on a slow phone, because the laptop hid a 3 s load.", 92),
            ("When a field has no visible label: give it an aria-label, because the screen reader read 'edit text'.", 99),
        ],
        dropped: &[
            "Run both shops a week",
            "Stack two coupons",
            "Sync the stock with the warehouse",
            "Offer pickup at the shop",
            "Read the total aloud on change",
        ],
        changed: &[
            "Round each price down to the cent",
            "Retry a payment that timed out",
            "Round tax per line",
            "Hold a payment with a high score",
            "Model the cart",
            "Cut the page to one request",
        ],
        live_from: 15,
        live_done: &[4, 2],
        open_from: Some(17),
        quiet: false,
        finish: false,
        start_after: 0,
    },
    Plan {
        title: "Merge the old leads into the contacts",
        project: "crm",
        steps: "Link each lead to its contact\n  Match a lead to a contact by email\n  Match the rest by phone\n  List the leads with no match\nMove the notes of each lead to its contact\n  Copy the notes\n  Keep each note's date\nRemove the old leads table",
        patterns: &[("When two records match by email and phone: trust the email, because phones were shared by a family.", 1)],
        dropped: &[],
        changed: &[],
        live_from: 0,
        live_done: &[3, 1],
        open_from: Some(2),
        quiet: false,
        finish: false,
        start_after: 0,
    },
    Plan {
        title: "Add a senior tier to the price list",
        project: "shop",
        steps: "Charge a senior 80% of the price\nShow the tier on the receipt\n  Add the tier's line to the receipt\n  Print the badge\nKeep the old discount banner\nShow the tier in the frame\n  Add the badge to the frame\n  Show the senior price\nRound the price down",
        patterns: &[
            ("When you price a senior: pay 80% of the price, rounded down to the cent, because the receipt must match the shelf.", 0),
            ("When you show a price: use the shop's currency, because the receipt showed dollars.", 3),
            ("When a badge is drawn: use the tier's colour from the theme, because a hard-coded green failed in dark mode.", 6),
        ],
        dropped: &["Keep the old discount banner"],
        changed: &["Charge a senior 80% of the price"],
        live_from: 3,
        live_done: &[1],
        open_from: Some(4),
        quiet: false,
        finish: false,
        start_after: 0,
    },
    small("Move the blog to the new theme", "web", "Port the post layout\n  Port the header\n  Port the footer\nPort the archive page\n  List posts by month\n  Paginate the archive", (1, &[0], true, false, 0)),
    small("Rotate the database passwords", "ops", "Rotate the primary's password\n  Write the new secret\n  Restart the readers\nRotate the replica's password\n  Write the replica's secret\n  Restart the replica's readers", (1, &[1], true, false, 0)),
    small("Speed up the image uploads", "web", "Resize images in the browser\n  Read the file\n  Scale it to 2000 px\nUpload in parts\n  Split the file\n  Retry a failed part\n  Join the parts", (1, &[1], false, false, 0)),
    small("Fix the flaky login test", "shop", "Find why the login test fails\n  Run it a hundred times\n  Read the failing logs\nWait for the session cookie\n  Wait on the cookie, not a timer", (1, &[1], false, true, 0)),
    small("Add dark mode to the dashboard", "dashboard", "Pick the dark palette\n  List the colours\n  Check their contrast\nSwitch the theme by the system\n  Read the system setting\n  Store the person's choice", (1, &[0], true, false, 0)),
    small("Write the quarterly sales report", "crm", "Pull the sales by region\n  Query the regions\n  Sum each quarter\nChart the sales\n  Draw a bar a region", (1, &[0], true, false, 0)),
    small("Upgrade the web framework", "web", "Read the upgrade guide\nBump the framework\n  Fix the router's breaking change\n  Fix the forms' breaking change\n  Run the whole suite", (1, &[1], true, false, 0)),
    Plan {
        title: "Archive the old orders",
        project: "shop",
        steps: "Copy the orders before 2022 to cold storage\n  Export them\n  Check the counts match\nDelete the copied orders",
        patterns: &[],
        dropped: &[],
        changed: &[],
        live_from: 0,
        live_done: &[1, 0],
        open_from: Some(1),
        quiet: false,
        finish: false,
        start_after: 14,
    },
];

/// A small trail of no patterns: its title, project and steps, and from which top step it plays live, how
/// many steps of each it closes then, whether it is quiet, whether it finishes, and when it starts.
const fn small(
    title: &'static str,
    project: &'static str,
    steps: &'static str,
    (live_from, live_done, quiet, finish, start_after): (
        usize,
        &'static [usize],
        bool,
        bool,
        usize,
    ),
) -> Plan {
    Plan {
        title,
        project,
        steps,
        patterns: &[],
        dropped: &[],
        changed: &[],
        live_from,
        live_done,
        open_from: None,
        quiet,
        finish,
        start_after,
    }
}

/// The steps of the biggest trail, one a line, nested by two spaces.
const CHECKOUT: &str = "Model the cart
  Add the cart table
  Add the line item table
  Add up the cart total in cents
  Keep a cart for a week
  Merge a guest cart on sign-in
Price the items
  Read prices from the shelf
  Apply the senior tier
  Apply the bulk discount
  Round each price down to the cent
  Show prices in the shop's currency
  Cache the price list
Take the address
  Add the address form
  Check the postcode
  Suggest the address as it is typed
  Keep the last address
  Show an error under each field
Sign the buyer in
  Add the sign-in form
  Mail a sign-in link
  Keep a buyer signed in a month
  Sign out on every device
  Lock an account after ten tries
Pick the delivery
  List the delivery options
  Price each option by weight
  Show the delivery date
  Offer pickup at the shop
Take the payment
  Add the card form
  Keep the card in the provider's frame
    Load the frame
    Send no card number to the server
  Charge the card
  Handle a declined card
  Retry a payment that timed out
  Keep the provider's receipt id
Send the receipt
  Draw the receipt
  Add the tier's line to the receipt
  Mail the receipt
  Print the receipt at the till
Keep the stock right
  Hold the stock while paying
  Free the stock of an abandoned cart
  Warn when an item runs out
  Sync the stock with the warehouse
  Count the stock each night
Apply coupons
  Add the coupon field
  Check a coupon's dates
  Stack two coupons
  Limit a coupon to one use
  Show the saving on the receipt
Charge tax
  Read the tax rate by region
  Add tax to each line
  Round tax per line
  Show tax on the receipt
  Report tax each month
Write the order history
  List a buyer's orders
  Filter orders by year
  Reorder from an old order
  Download an invoice
Track the order
  Add the order page
  Mail each change of state
  Show the parcel's tracking link
  Let a buyer cancel before shipping
Refund an order
  Refund the card
  Return the stock
  Mail the refund
  Refund part of an order
    Pick the lines to refund
    Refund their tax
Guard against fraud
  Score each payment
  Hold a payment with a high score
  Let staff release a held payment
  Block a card after three declines
Sync with the old shop
  Read the old orders
  Map the old products
  Run both shops a week
Make the checkout fast
  Measure each step of the checkout
  Load the card frame early
  Cache the delivery options
  Cut the page to one request
  Index the order tables
    Index orders by buyer
    Index lines by order
  Test the checkout under load
  Keep the page under one second
Make the checkout accessible
  Label each field
  Move focus to the first error
  Read the total aloud on change
  Check the contrast of each button
  Test with a screen reader
Translate the checkout
  Pull each string into the catalogue
  Translate to German
  Translate to Polish
  Format prices by locale
Write the checkout's docs
  Document the cart API
  Document the payment flow
  Document the refund flow
  Add a runbook for a provider outage
Launch the checkout
  Turn it on for staff
  Turn it on for 5% of buyers
  Watch the error rate a day
  Turn it on for everyone
Remove the old checkout";
