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
        Kind::of(30_078),
        None,
        "NIP-78 application data, which we never write"
    );
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
    assert_eq!(u16::from(Kind::Comment), 1_111, "NIP-22");
    assert_eq!(u16::from(Kind::RelayAuth), 22_242, "NIP-42");
    assert_eq!(u16::from(Kind::BlossomAuth), 24_242, "BUD-01");
    assert_eq!(u16::from(Kind::HttpAuth), 27_235, "NIP-98");
    assert_eq!(u16::from(Kind::ServerInvitation), 28_935, "NIP-43");
    assert_eq!(u16::from(Kind::Leave), 28_936, "NIP-43");
    assert_eq!(u16::from(Kind::Inbox), 10_050, "NIP-17");
    assert_eq!(u16::from(Kind::Relays), 10_002, "NIP-65");
    assert_eq!(u16::from(Kind::Article), 30_023, "NIP-23");
}

#[test]
/// A mark of ours is read latest-per-key by its `d`, and takes a number in NIP-01's addressable range so
/// that any client reads it the same way.
fn every_mark_of_ours_sits_in_the_addressable_range() {
    for kind in [
        Kind::Article,
        Kind::Record,
        Kind::Invitation,
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

#[test]
/// A mark whose number a NIP assigned takes that number, wherever it sits. Every record travels inside a
/// gift wrap (`spec/network.md`), so no relay reads the number and none of them retains by it — which
/// leaves this build's own fold the only thing addressing by `d`.
fn a_mark_a_nip_named_keeps_the_nip_s_number_and_is_still_folded_by_d() {
    for kind in [Kind::Comment, Kind::ServerInvitation] {
        assert!(
            !(30_000..40_000).contains(&u16::from(kind)),
            "{kind:?} would not need this test inside the addressable range"
        );
    }
}
