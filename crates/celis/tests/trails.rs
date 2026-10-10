//! CIP-14, "Trails".

mod support;

use support::{add, objects, Repo};

/// A reader counts a step with no `under` as a trail, and a step under one as no trail
/// (`spec/trails/14-trails.md`, "Trails").
#[test]
fn a_reader_counts_a_step_with_no_under_as_a_trail() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "senior tier"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "the price"]);
    let read = objects(&repo, &["read"]);
    let trails: Vec<&str> = read
        .iter()
        .filter_map(|v| v.get("trail")?["id"].as_str())
        .collect();
    let steps: Vec<&str> = read
        .iter()
        .filter_map(|v| v.get("step")?["id"].as_str())
        .collect();
    assert_eq!(trails, [trail.as_str()]);
    assert_eq!(steps, [part.as_str()]);
}

/// A reader counts a step in the tree of the trail at the top of its `under` chain
/// (`spec/trails/14-trails.md`, "Trails").
#[test]
fn a_step_counts_in_the_tree_of_the_trail_at_the_top_of_its_chain() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "senior tier"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "the price"]);
    let deep = add(&repo, &["--under", &part[..8], "--", "the rounding"]);
    let other = add(&repo, &["--", "other"]);
    let ids = |id: &str| -> Vec<String> {
        objects(&repo, &["tree", id])
            .iter()
            .filter_map(|v| Some(v.get("step")?["id"].as_str()?.to_string()))
            .collect()
    };
    assert_eq!(ids(&trail), [part, deep]);
    assert!(ids(&other).is_empty());
}
