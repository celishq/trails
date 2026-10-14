use super::*;

/// A `done` carries its proof and what changed from the plan, and reads back the same
/// (`spec/03-steps.md`, "Closing").
#[test]
fn a_done_carries_its_proof() {
    let account = Account::new(&[1u8; 32]).unwrap();
    let done = Mark::Done {
        proof: "proof".into(),
        changed: Some("changed".into()),
    };
    let note = done.record(&account, "p", 10).unwrap();
    assert_eq!(Mark::read(&note), Some(("p".to_string(), done)));
}
