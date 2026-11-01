use super::Meta;

#[test]
fn a_meta_prints_its_steps_and_time_and_skips_what_it_has_not() {
    let meta = |done, all, worked| Meta { done, all, worked }.to_string();
    assert_eq!(meta(3, 5, Some(126)), "3/5 steps · 2m 6s");
    assert_eq!(meta(1, 1, None), "1/1 step");
    assert_eq!(meta(0, 0, Some(9)), "9s");
    assert_eq!(meta(0, 0, None), "");
}
