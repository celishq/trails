use super::*;
use ratatui::style::Modifier;

/// The keys that lead each part are bold, what they do is plain, and the whole line is dim, so a key
/// stands apart by its weight alone.
#[test]
fn the_keys_are_bold_and_the_words_plain() {
    let line = legend("↑↓ navigate · enter or esc close");
    let parts: Vec<(&str, bool)> = line
        .spans
        .iter()
        .map(|s| {
            (
                s.content.as_ref(),
                s.style.add_modifier.contains(Modifier::BOLD),
            )
        })
        .collect();
    assert_eq!(
        parts,
        [
            ("↑↓", true),
            (" navigate", false),
            (" · ", false),
            ("enter", true),
            (" or", false),
            (" esc", true),
            (" close", false),
        ]
    );
    assert!(line
        .spans
        .iter()
        .all(|s| s.style.add_modifier.contains(Modifier::DIM)));
    assert_eq!(line.to_string(), "↑↓ navigate · enter or esc close");
}

/// A part that starts with no key, like `type to find`, stays plain.
#[test]
fn a_part_with_no_key_stays_plain() {
    let line = legend("type to find · / find");
    assert!(!line.spans[0].style.add_modifier.contains(Modifier::BOLD));
    assert_eq!(line.spans[0].content, "type to find");
    assert!(line.spans[2].style.add_modifier.contains(Modifier::BOLD));
}
