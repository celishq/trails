use super::*;

/// A time `took` shows changes with its smallest unit: each second up to an hour, then each minute, then
/// each hour.
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

/// The month of a time starts at the local midnight of its first day.
#[test]
fn a_month_starts_at_the_midnight_of_its_first_day() {
    let at = 1_800_000_000;
    let start = month(at);
    assert!(date(start).ends_with("-01"), "{}", date(start));
    assert_eq!(midnight(start), start);
    assert_eq!(&date(start)[..7], &date(at)[..7]);
}
