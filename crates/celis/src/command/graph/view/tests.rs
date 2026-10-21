use super::*;
use crate::command::graph::tests::Bench;

/// A view inside the trail `trail` of `ids`, its cards one a row in their order.
fn inside(trail: &str, ids: &[&str]) -> View {
    let mut view = View {
        mode: Some(Mode::Trail),
        trail: Some(trail.to_string()),
        trails: vec![trail.to_string(), "other".to_string()],
        ..View::default()
    };
    view.cards = ids
        .iter()
        .enumerate()
        .map(|(n, id)| (id.to_string(), 2, n as u16 * 6))
        .collect();
    view
}

/// Esc steps back one level at a time: from a row to its card, from an opened card to the card closed,
/// from a card to none, and from none to all trails.
#[test]
fn esc_steps_back_one_level() {
    let bench = Bench::new();
    let steps = bench.trails.steps();
    let mut view = inside("t", &["a", "b"]);
    view.selected = Some("a".into());
    view.opened.insert("a".into());
    view.row = Some("a1".into());
    view.press(Key::Escape, &steps);
    assert_eq!(
        (view.row.as_deref(), view.opened.contains("a")),
        (None, true)
    );
    view.press(Key::Escape, &steps);
    assert_eq!(
        (view.selected.as_deref(), view.opened.contains("a")),
        (Some("a"), false)
    );
    view.press(Key::Escape, &steps);
    assert_eq!(view.selected, None);
    view.press(Key::Escape, &steps);
    assert_eq!((view.mode(), view.pick.as_deref()), (Mode::All, Some("t")));
}

/// `a` goes to all trails from any depth with the trail picked, and tab to the next trail.
#[test]
fn a_and_tab_leave_the_trail() {
    let bench = Bench::new();
    let steps = bench.trails.steps();
    let mut view = inside("t", &["a"]);
    view.press(Key::Tab, &steps);
    assert_eq!(view.trail.as_deref(), Some("other"));
    view.selected = Some("a".into());
    view.row = Some("a1".into());
    view.press(Key::All, &steps);
    assert_eq!(
        (view.mode(), view.pick.as_deref(), view.row.as_deref()),
        (Mode::All, Some("other"), None)
    );
}

/// `r` opens the rules over the screen with the newest selected, ↓ moves to the next, and esc closes
/// them.
#[test]
fn r_opens_the_rules_and_esc_closes_them() {
    let bench = Bench::new();
    let steps = bench.trails.steps();
    let mut view = inside("t", &["a"]);
    view.learned = vec!["new".into(), "old".into()];
    view.press(Key::Char('r'), &steps);
    assert_eq!((view.rules, view.rule.as_deref()), (true, Some("new")));
    view.press(Key::Down, &steps);
    assert_eq!(view.rule.as_deref(), Some("old"));
    view.press(Key::Escape, &steps);
    assert!(!view.rules);
}

/// `/` searches all trails: every key types, `q` too, erase takes one off, and esc picks again the
/// trail picked before.
#[test]
fn a_search_types_every_key() {
    let bench = Bench::new();
    let steps = bench.trails.steps();
    let mut view = inside("t", &["a"]);
    view.press(Key::Char('/'), &steps);
    assert_eq!((view.mode(), view.query.as_deref()), (Mode::All, Some("")));
    for key in [Key::Quit, Key::Char('u'), Key::All, Key::Erase] {
        view.press(key, &steps);
    }
    assert_eq!(view.query.as_deref(), Some("qu"));
    assert!(view.finds("Quarterly sales", "crm"));
    assert!(!view.finds("Build the checkout", "shop"));
    view.press(Key::Escape, &steps);
    assert_eq!(
        (view.query.as_deref(), view.pick.as_deref()),
        (None, Some("t"))
    );
}

/// The key line is one for all trails and one for inside a trail, esc last as back and no quit.
#[test]
fn the_key_line_ends_with_esc_and_names_no_quit() {
    let mut view = inside("t", &["a"]);
    assert!(view.keys().ends_with("esc back"));
    view.selected = Some("a".into());
    assert_eq!(view.keys(), inside("t", &["a"]).keys());
    view.show(None);
    assert_eq!(view.keys(), "↑↓ navigate · enter open · / find · r rules");
    assert!(!view.keys().contains("quit"));
}

/// In an opened card, ↓ steps into its rows, → unfolds a nesting and then steps into it, and ← steps
/// out to its parent.
#[test]
fn the_arrows_walk_an_opened_card() {
    let mut bench = Bench::new();
    let trail = bench.step("shop", None, "the trail");
    let card = bench.step("shop", Some(&trail), "the card");
    let row = bench.step("shop", Some(&card), "the row");
    let deep = bench.step("shop", Some(&row), "the deep row");
    let steps = bench.trails.steps();
    let mut view = inside(&trail, &[&card]);
    view.selected = Some(card.clone());
    view.opened.insert(card.clone());
    view.listed = vec![row.clone()];
    view.press(Key::Down, &steps);
    assert_eq!(view.row.as_deref(), Some(row.as_str()));
    view.press(Key::Right, &steps);
    assert!(view.unfolded.contains(&row));
    view.press(Key::Right, &steps);
    assert_eq!(view.row.as_deref(), Some(deep.as_str()));
    view.press(Key::Left, &steps);
    assert_eq!(view.row.as_deref(), Some(row.as_str()));
}

/// With no trail named the view opens on All, even with one trail of today; with one named, on its
/// graph.
#[test]
fn the_view_opens_on_all_unless_a_trail_is_named() {
    let mut bench = Bench::new();
    let trail = bench.step("shop", None, "the one trail");
    bench.step("shop", Some(&trail), "a step");
    let (all, named) = crate::progress::Model::of(&bench.trails, |m| {
        let trails = super::super::folders(m.steps);
        assert_eq!(trails.len(), 1);
        let all = View::new(&trails, None).mode();
        let named = View::new(&trails, Some(&trail)).mode();
        (all, named)
    });
    assert_eq!((all, named), (Mode::All, Mode::Trail));
}
