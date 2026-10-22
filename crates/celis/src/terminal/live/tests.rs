use super::*;

/// Keys wait out the rest of a frame since the screen last wrote one, so a held key draws at most one
/// frame each 16 ms; with no frame written, or one a frame ago, they wait none.
#[test]
fn keys_wait_out_the_rest_of_a_frame() {
    assert_eq!(pace(None), Duration::ZERO);
    assert_eq!(pace(Some(Instant::now() - FRAME)), Duration::ZERO);
    let rest = pace(Some(Instant::now()));
    assert!(rest > FRAME / 2 && rest <= FRAME, "{rest:?}");
}
