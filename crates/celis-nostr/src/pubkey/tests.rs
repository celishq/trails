use super::*;

/// fiatjaf's key, from a note that has been on the network since 2022.
const HEX: &str = "3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d";

#[test]
fn a_key_reads_the_same_in_hex_and_as_an_npub() {
    let key = Pubkey::parse(HEX).unwrap();
    assert_eq!(key.hex(), HEX);
    assert_eq!(
        key.npub(),
        "npub180cvv07tjdrrgpa0j7j7tmnyl2yr6yr7l8j4s3evf6u64th6gkwsyjh6w6",
        "the npub other clients show for this key"
    );
    assert_eq!(Pubkey::parse_npub(&key.npub()).unwrap(), key);
}

#[test]
fn a_key_that_is_not_on_the_curve_is_refused() {
    assert!(Pubkey::new([0u8; 32]).is_err());
    assert!(Pubkey::parse("not hex").is_err());
    assert!(Pubkey::parse(&HEX[..62]).is_err(), "too short");
    assert!(
        Pubkey::parse_npub("nsec1vl029mgpspedva04g90vltkh6fvh240zqtv9k0t9af8935ke9laqsnlfe5")
            .is_err()
    );
}

#[test]
fn a_key_survives_a_json_round_trip_as_hex() {
    let key = Pubkey::parse(HEX).unwrap();
    let json = serde_json::to_string(&key).unwrap();
    assert_eq!(json, format!("\"{HEX}\""));
    assert_eq!(serde_json::from_str::<Pubkey>(&json).unwrap(), key);
}
