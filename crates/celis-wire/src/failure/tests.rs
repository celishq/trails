use super::*;

#[test]
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

/// (`spec/network.md`, "What a refusal means")
#[test]
fn a_relays_word_says_what_is_left_to_do_only_when_it_is_the_whole_prefix() {
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

#[test]
fn only_a_404_says_the_host_does_not_hold_it() {
    let refused = |status| {
        Failure::new(
            "/abc",
            Why::Refused {
                status,
                body: String::new(),
                after: None,
            },
        )
    };
    assert!(refused(404).missing());
    assert!(!refused(403).missing());
    assert!(!Failure::new("/abc", Why::Unreachable).missing());
}
