use super::*;

/// A time `took` shows changes with its smallest unit: each second up to an hour, then each minute, then
/// each hour (`spec/trails/04-trails.md`, "Watching").
#[test]
fn a_time_changes_with_its_smallest_unit() {
    assert_eq!(stride(0), 1);
    assert_eq!(stride(3_599), 1);
    assert_eq!(stride(3_600), 60);
    assert_eq!(stride(3_650), 10);
    assert_eq!(stride(86_399), 1);
    assert_eq!(stride(86_400), 3_600);
    assert_eq!(stride(86_400 + 3_000), 600);
}
