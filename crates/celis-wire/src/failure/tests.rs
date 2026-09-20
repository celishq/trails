use super::*;

#[test]
/// A refusal of the request is settled; a host that is busy, over its rate, or failing is asking to be
/// asked again, and says how long where it can.
fn a_status_says_what_is_left_to_do() {
    assert_eq!(Verdict::of_status(400, None), Verdict::Settled);
    assert_eq!(Verdict::of_status(402, None), Verdict::Settled);
    assert_eq!(Verdict::of_status(404, None), Verdict::Settled);
    assert_eq!(
        Verdict::of_status(429, Some(7)),
        Verdict::Again { after: Some(7) }
    );
    for status in [500, 502, 503, 504] {
        assert_eq!(
            Verdict::of_status(status, None),
            Verdict::Again { after: None },
            "{status}"
        );
    }
}

#[test]
/// The prefix is the whole of the reading (`spec/fixtures/refusals.json`): three words are about the
/// note, and every other word, a word inside the sentence, or no word is the relay about itself.
fn a_relays_word_says_what_is_left_to_do() {
    for word in [
        "invalid: bad sig",
        "restricted: not a member",
        "pow: 20 bits",
    ] {
        assert_eq!(Verdict::of_word(word), Verdict::Settled, "{word}");
    }
    for word in [
        "blocked: over allowance",
        "rate-limited: slow down",
        "error: could not save: invalid: about another note",
        "something new",
        "",
    ] {
        assert_eq!(
            Verdict::of_word(word),
            Verdict::Again { after: None },
            "{word}"
        );
    }
}

/// Every prefix in `spec/fixtures/refusals.json`, which the host writes against — so the table here
/// and the words the host writes cannot drift apart without failing. `took` is not a verdict: a relay
/// that has the note refused nothing, and what reads that is the offer.
#[test]
fn every_prefix_the_host_writes_carries_the_verdict_the_fixture_states() {
    let stated: serde_json::Value =
        serde_json::from_str(include_str!("../../../../spec/fixtures/refusals.json")).unwrap();
    let prefixes = stated["prefixes"].as_array().unwrap();
    assert!(!prefixes.is_empty());
    for stated in prefixes {
        let prefix = stated["prefix"].as_str().unwrap();
        let expected = match stated["offer"].as_str().unwrap() {
            "took" => continue,
            "refused" => Verdict::Settled,
            "unsettled" => Verdict::Again { after: None },
            other => panic!("the fixture names an offer this build does not read: {other}"),
        };
        assert_eq!(
            Verdict::of_word(&format!("{prefix}: what the host said")),
            expected,
            "`{prefix}:`"
        );
    }
}

#[test]
fn a_failure_reads_its_verdict_off_what_became_of_it() {
    assert_eq!(
        Failure::new("/ai", Why::Unreachable).verdict(),
        Verdict::Unreached
    );
    assert_eq!(
        Failure::new(
            "/ai",
            Why::Unreadable {
                detail: "not json".into()
            }
        )
        .verdict(),
        Verdict::Again { after: None }
    );
    let spent = Failure::new(
        "/ai",
        Why::Refused {
            status: 402,
            body: String::new(),
            after: None,
        },
    );
    assert!(spent.settled());
    let busy = Failure::new(
        "/ai",
        Why::Refused {
            status: 503,
            body: String::new(),
            after: Some(3),
        },
    );
    assert!(!busy.settled());
    assert_eq!(busy.verdict(), Verdict::Again { after: Some(3) });
}
