use super::*;
use celis_nostr::{Account, Note};
use celis_trails::{Anchor, Rules, StepId, StepRecord, Steps};

fn step(under: Option<&Note>, at: u64, content: &str) -> Note {
    StepRecord {
        anchor: Anchor {
            project: "p".into(),
            name: "repo".into(),
        },
        under: under.map(|n| StepId::new(&n.id)),
        replaces: None,
        uses: Vec::new(),
        places: Vec::new(),
        check: None,
        session: None,
        content: content.into(),
    }
    .record(&Account::new(&[1u8; 32]).unwrap(), at)
    .unwrap()
}

/// The selected card keeps the dim border of its state, and its mark and title show in bold bright
/// yellow.
#[test]
fn a_selected_card_shows_in_bold_bright_yellow_text() {
    let now = 1_800_000_000;
    let trail = step(None, now - 100, "trail");
    let first = step(Some(&trail), now - 90, "first");
    let page = step(Some(&trail), now - 80, "page");
    let records = [trail.clone(), first, page.clone()];
    let steps = Steps::new(&records, now);
    let rules = Rules::new(&records);
    let events = Events::of(steps.get(&trail.id).unwrap(), &rules);
    let diagram = Diagram {
        head: steps.get(&trail.id).unwrap(),
        events: &events,
        shown: events.all.len(),
        selected: Some(&page.id),
        spin: None,
    };
    let mut buf = Buffer::empty(Rect::new(0, 0, 80, 12));
    let areas = diagram.render(buf.area, &mut buf, &Cell::new(0));
    let card = areas[&page.id];
    let corner = &buf[(card.x, card.y)];
    assert_eq!(corner.symbol(), "┌");
    assert!(corner.modifier.contains(Modifier::DIM));
    for x in [card.x + 2, card.x + 4] {
        let cell = &buf[(x, card.y + 1)];
        assert_eq!(
            (cell.fg, cell.modifier),
            (Color::LightYellow, Modifier::BOLD)
        );
    }
}
