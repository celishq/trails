use super::Cells;
use crate::{Cell, Mask};
use celis_nostr::Account;
use celis_nostr::Note;
use celis_store::Anchor;

fn mask(cell: &Cell, account: &Account, at: u64, on: bool, anchor: Option<Anchor>) -> Note {
    let cell = cell.clone();
    let mask = Mask {
        cell,
        on,
        anchor,
        at,
    };
    mask.record(account).unwrap()
}

fn anchor(project: &str) -> Anchor {
    Anchor {
        project: project.to_string(),
        name: project.to_string(),
    }
}

/// A project mask wins over a newer `me` mask, and a `me` mask holds where no project mask is
/// (`spec/09-cells.md`, "Masks").
#[test]
fn a_project_mask_wins_over_a_me_mask() {
    let account = Account::new(&[1u8; 32]).unwrap();
    let cell = Cell {
        name: "west".to_string(),
        author: account.pubkey().hex(),
    };
    let records = vec![
        mask(&cell, &account, 1, true, Some(anchor("p"))),
        mask(&cell, &account, 2, false, None),
        mask(&cell, &account, 3, true, Some(anchor("q"))),
        mask(&cell, &account, 4, false, Some(anchor("q"))),
    ];
    let cells = Cells::new(&records);
    assert!(cells.is_on(&cell, "p"));
    assert!(!cells.is_on(&cell, "q"));
    assert!(!cells.is_on(&cell, "r"));
    let on = mask(&cell, &account, 5, true, None);
    let cells = Cells::new(&[records, vec![on]].concat());
    assert!(cells.is_on(&cell, "r"));
    assert!(!cells.is_on(&cell, "q"));
}

/// A cell with no mask is off (`spec/09-cells.md`, "Masks").
#[test]
fn a_cell_with_no_mask_is_off() {
    let cell = Cell {
        name: "west".to_string(),
        author: "ab".to_string(),
    };
    assert!(!Cells::new(&[]).is_on(&cell, "p"));
}

/// `newest` gives the newest mask of a cell in one project, or of every project.
#[test]
fn newest_gives_the_newest_mask_of_the_project() {
    let account = Account::new(&[1u8; 32]).unwrap();
    let cell = Cell {
        name: "west".to_string(),
        author: account.pubkey().hex(),
    };
    let records = vec![
        mask(&cell, &account, 5, true, Some(anchor("p"))),
        mask(&cell, &account, 9, false, None),
        mask(&cell, &account, 7, false, Some(anchor("p"))),
    ];
    let cells = Cells::new(&records);
    let at = |project| cells.newest(&cell, project).map(|m| m.at);
    assert_eq!(at(Some("p")), Some(7));
    assert_eq!(at(None), Some(9));
    assert_eq!(at(Some("q")), None);
}
