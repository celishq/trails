use super::Cells;
use crate::Cell;
use celis_nostr::Account;
use celis_store::Anchor;

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
        cell.mask(&account, 1, true, Some(&anchor("p"))).unwrap(),
        cell.mask(&account, 2, false, None).unwrap(),
        cell.mask(&account, 3, true, Some(&anchor("q"))).unwrap(),
        cell.mask(&account, 4, false, Some(&anchor("q"))).unwrap(),
    ];
    let cells = Cells::new(&records);
    assert!(cells.is_on(&cell, "p"));
    assert!(!cells.is_on(&cell, "q"));
    assert!(!cells.is_on(&cell, "r"));
    let on = cell.mask(&account, 5, true, None).unwrap();
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
