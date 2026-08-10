use super::*;
use std::collections::BTreeSet;

#[test]
fn every_kind_reads_back_from_its_own_number() {
    for kind in ALL {
        assert_eq!(Kind::of(u16::from(kind)), Some(kind));
        assert_eq!(kind.to_string(), u16::from(kind).to_string());
    }
}

#[test]
fn no_two_kinds_claim_one_number() {
    let numbers: BTreeSet<u16> = ALL.into_iter().map(u16::from).collect();
    assert_eq!(numbers.len(), ALL.len());
}

#[test]
fn a_kind_this_build_does_not_know_is_none() {
    assert_eq!(Kind::of(1), None, "a plain note");
    assert_eq!(
        Kind::of(30_023),
        None,
        "NIP-23 long-form, which we never write"
    );
    assert_eq!(Kind::of(0), None);
}

#[test]
/// The numbers other clients already write, pinned against the NIPs that assign them — so a typo here reads
/// as somebody else's kind rather than failing only where two of our crates disagree.
fn the_spec_assigned_numbers_are_what_the_nips_say() {
    assert_eq!(u16::from(Kind::Deletion), 5, "NIP-09");
    assert_eq!(u16::from(Kind::Reaction), 7, "NIP-25");
    assert_eq!(u16::from(Kind::Seal), 13, "NIP-59");
    assert_eq!(u16::from(Kind::Wrap), 1_059, "NIP-59");
    assert_eq!(u16::from(Kind::Blob), 1_063, "NIP-94");
    assert_eq!(u16::from(Kind::RelayAuth), 22_242, "NIP-42");
    assert_eq!(u16::from(Kind::BlossomAuth), 24_242, "BUD-01");
    assert_eq!(u16::from(Kind::HttpAuth), 27_235, "NIP-98");
    assert_eq!(u16::from(Kind::Leave), 28_936, "NIP-43");
}

#[test]
/// A mark is read latest-per-key by its `d`, which is what NIP-01's addressable range means to anything
/// reading a record.
fn every_mark_sits_in_the_addressable_range() {
    for kind in [
        Kind::Record,
        Kind::Comment,
        Kind::Invitation,
        Kind::ServerInvitation,
        Kind::Server,
        Kind::Membership,
        Kind::Epoch,
        Kind::Keyring,
    ] {
        assert!(
            (30_000..40_000).contains(&u16::from(kind)),
            "{kind:?} is outside the addressable range"
        );
    }
}
