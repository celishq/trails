use super::*;
use crate::command::read::graph::tests::Bench;
use celis_store::Store;
use celis_trails::TrailStore;

/// The titles of the trails `folders` gives for `trails`, in order.
fn titles(trails: &Store) -> Vec<String> {
    let steps = trails.steps();
    folders(&steps)
        .iter()
        .map(|t| t.title().to_string())
        .collect()
}

/// Each folder keeps the place its oldest trail gives it, and a new trail goes first in its own folder
/// without moving any folder.
#[test]
fn a_new_trail_goes_first_in_its_folder_and_moves_no_folder() {
    let mut bench = Bench::new();
    let shop = bench.step("shop", None, "shop one");
    let a = bench.step("shop", Some(&shop), "a");
    bench.start(&a);
    let crm = bench.step("crm", None, "crm one");
    let b = bench.step("crm", Some(&crm), "b");
    bench.start(&b);
    assert_eq!(titles(&bench.trails), ["shop one", "crm one"]);
    let two = bench.step("shop", None, "shop two");
    let c = bench.step("shop", Some(&two), "c");
    bench.start(&c);
    assert_eq!(titles(&bench.trails), ["shop two", "shop one", "crm one"]);
}
