//! A record id, whole or by the first 8 characters a text line shows of it.

use celis_wire::Note;

/// The first 8 characters of `id`.
pub fn short(id: &str) -> String {
    id.chars().take(8).collect()
}

/// The note of `notes` whose id is `id`, or the only one whose id starts with `id` when `id` is 8
/// characters.
pub fn find<'a>(notes: impl IntoIterator<Item = &'a Note>, id: &str) -> Option<&'a Note> {
    let notes: Vec<&'a Note> = notes.into_iter().collect();
    if let Some(note) = notes.iter().find(|n| n.id == id) {
        return Some(note);
    }
    let prefixed: Vec<&'a Note> = match id.chars().count() {
        8 => notes.into_iter().filter(|n| n.id.starts_with(id)).collect(),
        _ => Vec::new(),
    };
    match prefixed[..] {
        [note] => Some(note),
        _ => None,
    }
}
