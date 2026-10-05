//! CIP-11, "Commands".

mod support;
use serde_json::Value;
use std::process::Output;
use support::{agent, cairn, cairn_saying, err, labelled, now, out, rating, run, Repo, DAY};

fn write(repo: &Repo, env: &[(&str, &str)], args: &[&str]) -> Output {
    let mut all = vec!["write"];
    all.extend_from_slice(args);
    repo.run(&repo.path(), &all, env, "")
}

fn listed(repo: &Repo, args: &[&str]) -> Vec<Value> {
    let mut all = vec!["list", "--json"];
    all.extend_from_slice(args);
    let o = repo.run(&repo.path(), &all, &[], "");
    assert!(o.status.success(), "{}", err(&o));
    out(&o)
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

fn contents(repo: &Repo, args: &[&str]) -> Vec<String> {
    let mut all: Vec<String> = listed(repo, args)
        .iter()
        .map(|e| e["content"].as_str().unwrap().to_string())
        .collect();
    all.sort();
    all
}

/// With `$CELIS_SESSION` set a command signs as that session's agent, without it as the account
/// (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn with_a_session_a_command_signs_as_that_sessions_agent() {
    let repo = Repo::new();
    let one = [("CELIS_SESSION", "s1")];
    let two = [("CELIS_SESSION", "s2")];
    for (env, text) in [(&one[..], "a"), (&one, "b"), (&two, "c"), (&[], "d")] {
        let args = ["finding", "--else", text, "--", text];
        let o = write(&repo, env, &args);
        assert!(o.status.success(), "{}", err(&o));
    }
    let all = listed(&repo, &[]);
    let agent = |text: &str| all.iter().find(|e| e["content"] == text).unwrap()["agent"].clone();
    assert_eq!(agent("a"), agent("b"));
    assert_ne!(agent("a"), agent("c"));
    assert_ne!(agent("a"), agent("d"));
    assert_ne!(agent("c"), agent("d"));
}

/// `list` never prints a spot (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn list_never_prints_a_spot() {
    let repo = Repo::new();
    let by = agent(&repo, "a");
    let kept = cairn(&repo, &by, "finding", &[], now() - DAY);
    cairn(&repo, &by, "spot", &[("else", "E-SPOT")], now() - DAY);
    let all: Vec<Value> = listed(&repo, &[])
        .into_iter()
        .filter(|e| e.get("id").is_some())
        .collect();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0]["id"], kept.id.as_str());
    assert!(listed(&repo, &["--error", "E-SPOT"])
        .iter()
        .all(|e| e["label"] != "spot"));
}

/// With `--spot`, `write` takes the `else` of that spot, adds those the flags give, and
/// takes the first 8 characters of the spot's id (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn with_spot_write_takes_the_else_of_that_spot_and_adds_the_flags() {
    let repo = Repo::new();
    let by = agent(&repo, "a");
    let spot = cairn(&repo, &by, "spot", &[("else", "E-SPOT")], now() - DAY);
    let args = [
        "finding",
        "--spot",
        &spot.id[..8],
        "--else",
        "E-MORE",
        "--",
        "fix",
    ];
    let o = write(&repo, &[], &args);
    assert!(o.status.success(), "{}", err(&o));
    let id = out(&o).trim().to_string();
    let all = listed(&repo, &[]);
    let made = all.iter().find(|e| e["id"] == id.as_str()).unwrap();
    let names = |key: &str| {
        let mut v: Vec<String> = made[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap().to_string())
            .collect();
        v.sort();
        v
    };
    assert_eq!(names("else"), ["E-MORE", "E-SPOT"]);
}

/// `write` refuses a `--spot` that names no live spot, with `no-cairn` (`spec/cairns/11-cli.md`,
/// "Commands", "Output and errors").
#[test]
fn write_refuses_a_spot_that_names_no_live_spot() {
    let repo = Repo::new();
    let by = agent(&repo, "a");
    let kept = cairn(&repo, &by, "finding", &[], now() - DAY);
    for id in ["0".repeat(64), kept.id.clone(), kept.id[..8].to_string()] {
        let args = ["finding", "--spot", &id, "--else", "E", "--", "x"];
        let o = repo.run(
            &repo.path(),
            &[&["write", "--json"][..], &args].concat(),
            &[],
            "",
        );
        assert_eq!(o.status.code(), Some(1), "{id}: {}", err(&o));
        let object: Value = serde_json::from_str(err(&o).trim()).unwrap();
        assert_eq!(object["error"], "no-cairn");
    }
}

/// `list --error` is the error lookup and writes a spot (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn list_error_is_the_error_lookup_and_writes_a_spot() {
    let repo = Repo::new();
    agent(&repo, "s1");
    assert!(labelled(&repo, "spot").is_empty());
    run(&repo, "s1", &["list", "--error", "E-LOOKUP"]);
    let spots = labelled(&repo, "spot");
    assert_eq!(spots.len(), 1);
    assert_eq!(spots[0].tag("else"), Some("E-LOOKUP"));
}

/// `write` refuses a `--spot` that names an expired spot, with `no-cairn` (`spec/cairns/11-cli.md`,
/// "Commands").
#[test]
fn write_refuses_an_expired_spot() {
    let repo = Repo::new();
    let by = agent(&repo, "a");
    let spot = cairn(&repo, &by, "spot", &[("else", "E-SPOT")], now() - 15 * DAY);
    refused_spot(&repo, &spot.id);
}

/// `write` refuses a `--spot` that names a pulled spot, with `no-cairn` (`spec/cairns/11-cli.md`,
/// "Commands").
#[test]
fn write_refuses_a_pulled_spot() {
    let repo = Repo::new();
    let by = agent(&repo, "a");
    let spot = cairn(&repo, &by, "spot", &[("else", "E-SPOT")], now() - DAY);
    for session in ["b", "c"] {
        rating(&repo, &agent(&repo, session), &spot, false, now() - 3600);
    }
    refused_spot(&repo, &spot.id);
}

fn refused_spot(repo: &Repo, id: &str) {
    let args = [
        "write", "--json", "finding", "--spot", id, "--else", "E", "--", "x",
    ];
    let o = repo.run(&repo.path(), &args, &[], "");
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    let object: Value = serde_json::from_str(err(&o).trim()).unwrap();
    assert_eq!(object["error"], "no-cairn");
}

/// `rate` refuses an id no cairn in the silo has, with `no-cairn` (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn rate_refuses_an_unknown_id_with_no_cairn() {
    let repo = Repo::new();
    let id = "0".repeat(64);
    let o = repo.run(&repo.path(), &["rate", "--json", &id, "+"], &[], "");
    assert_eq!(o.status.code(), Some(1));
    let object: Value = serde_json::from_str(err(&o).trim()).unwrap();
    assert_eq!(object["error"], "no-cairn");
}

/// With `$CELIS_SESSION` set, `rate` and the spot an error lookup writes are signed by that session's
/// agent, not the account (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn with_a_session_rate_and_the_error_lookup_sign_as_that_sessions_agent() {
    let repo = Repo::new();
    let session = [("CELIS_SESSION", "s1")];
    let args = ["finding", "--else", "E-ONE", "--", "one"];
    let o = write(&repo, &session, &args);
    assert!(o.status.success(), "{}", err(&o));
    let o = write(&repo, &[], &["decision", "--", "two"]);
    assert!(o.status.success(), "{}", err(&o));
    let id = out(&o).trim().to_string();
    let all = listed(&repo, &[]);
    let signer = |text: &str| all.iter().find(|e| e["content"] == text).unwrap()["agent"].clone();
    let (agent_npub, account_npub) = (signer("one"), signer("two"));
    assert_ne!(agent_npub, account_npub);

    let rated = repo.run(&repo.path(), &["rate", &id, "+"], &session, "");
    assert!(rated.status.success(), "{}", err(&rated));
    run(&repo, "s1", &["list", "--error", "E-LOOKUP"]);

    let notes: Vec<celis_wire::Note> = std::fs::read_dir(repo.silo().join("shared"))
        .unwrap()
        .filter_map(|e| std::fs::read_to_string(e.unwrap().path()).ok())
        .filter_map(|t| serde_json::from_str(&t).ok())
        .collect();
    let rating = notes.iter().find(|n| n.kind == 7).unwrap();
    assert_eq!(rating.pubkey.npub(), agent_npub.as_str().unwrap());
    let spots = labelled(&repo, "spot");
    assert_eq!(spots.len(), 1);
    assert_eq!(spots[0].pubkey.npub(), agent_npub.as_str().unwrap());
}

/// `--days` takes a whole number from 1: `stats` and `log` refuse 0, abc, -1 and 1.5 as a usage error
/// (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn days_takes_a_whole_number_from_one() {
    let repo = Repo::new();
    for command in ["stats", "log"] {
        for days in ["0", "abc", "-1", "1.5"] {
            let o = repo.run(&repo.path(), &[command, "--days", days], &[], "");
            assert_eq!(o.status.code(), Some(2), "{command} {days}: {}", err(&o));
        }
        let o = repo.run(&repo.path(), &[command, "--days", "1"], &[], "");
        assert!(o.status.success(), "{command} 1: {}", err(&o));
    }
}

/// `list` takes no `--task`: it is a usage error like any unknown flag (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn list_refuses_task_as_a_usage_error() {
    let repo = Repo::new();
    let o = repo.run(&repo.path(), &["list", "--task", "x"], &[], "");
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
}

fn spot(repo: &Repo, by: &celis_cairns::Agent, error: &str, ago: u64) {
    cairn(repo, by, "spot", &[("else", error)], now() - ago);
}

fn unsolved(repo: &Repo, args: &[&str]) -> Vec<(String, u64)> {
    listed(repo, args)
        .iter()
        .filter_map(|o| Some((o["unsolved"].as_str()?.to_string(), o["spots"].as_u64()?)))
        .collect()
}

/// `list` prints every live cairn but spots, in reading order, with no limit
/// (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn list_must_print_every_live_cairn_but_spots_with_no_limit() {
    let repo = Repo::new();
    let by = agent(&repo, "a");
    let ids: Vec<String> = (0..12)
        .map(|i| cairn(&repo, &by, "finding", &[], now() - (20 - i) * 3600).id)
        .collect();
    cairn(&repo, &by, "spot", &[("else", "E-SPOT")], now() - 3600);
    let printed: Vec<String> = listed(&repo, &[])
        .iter()
        .filter_map(|e| Some(e["id"].as_str()?.to_string()))
        .collect();
    let newest_first: Vec<String> = ids.iter().rev().cloned().collect();
    assert_eq!(printed, newest_first);
}

/// `list` then prints a line for each unsolved error, most spots first, with no limit, and an error a
/// live cairn matches is not one (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn list_must_then_print_a_line_for_each_unsolved_error_most_spots_first() {
    let repo = Repo::new();
    let by = agent(&repo, "a");
    for (error, spots) in [("E-A", 4), ("E-B", 3), ("E-C", 2), ("E-D", 1), ("E-NET", 5)] {
        for n in 0..spots {
            spot(&repo, &by, error, 3600 + n * 60);
        }
    }
    cairn(&repo, &by, "finding", &[("else", "E-NET")], now() - DAY);
    let all = listed(&repo, &[]);
    assert_eq!(all[0]["label"], "finding");
    assert_eq!(
        unsolved(&repo, &[]),
        [
            ("E-A".to_string(), 4),
            ("E-B".to_string(), 3),
            ("E-C".to_string(), 2),
            ("E-D".to_string(), 1)
        ]
    );
}

/// Past 8000 bytes of content, `list` ends with `note: <n> cairns, about <bytes / 4> tokens`
/// (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn when_the_content_list_printed_holds_more_than_8000_bytes_list_ends_with_a_note() {
    let repo = Repo::new();
    let by = agent(&repo, "a");
    let long = "x".repeat(500);
    for i in 0..17 {
        cairn_saying(&repo, &by, &long, now() - (20 - i) * 3600);
    }
    let o = repo.run(&repo.path(), &["list"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    let last = out(&o).lines().last().unwrap().to_string();
    assert_eq!(last, "note: 17 cairns, about 2125 tokens");

    let few = Repo::new();
    let by = agent(&few, "a");
    cairn_saying(&few, &by, &long, now() - DAY);
    let o = few.run(&few.path(), &["list"], &[], "");
    assert!(!out(&o).contains("note:"), "{}", out(&o));
}

/// Exactly 8000 bytes of content print no note; one byte more prints it
/// (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn list_prints_the_note_only_past_8000_bytes() {
    let last_line = |extra: usize| {
        let repo = Repo::new();
        let by = agent(&repo, "a");
        for i in 0..8 {
            let size = if i == 0 { 1000 + extra } else { 1000 };
            cairn_saying(&repo, &by, &"x".repeat(size), now() - (20 - i) * 3600);
        }
        let o = repo.run(&repo.path(), &["list"], &[], "");
        assert!(o.status.success(), "{}", err(&o));
        out(&o).lines().last().unwrap().to_string()
    };
    assert!(!last_line(0).contains("note:"));
    assert_eq!(last_line(1), "note: 8 cairns, about 2000 tokens");
}

/// `list --error` prints only the live cairns whose `else` matches the error
/// (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn list_error_must_print_only_the_live_cairns_whose_else_matches() {
    let repo = Repo::new();
    let by = agent(&repo, "a");
    cairn_saying(&repo, &by, "none", now() - DAY);
    cairn(
        &repo,
        &by,
        "finding",
        &[("else", "ConnectionError: E-NET")],
        now() - DAY,
    );
    cairn(&repo, &by, "decision", &[("else", "E-OTHER")], now() - DAY);
    assert_eq!(contents(&repo, &["--error", "E-NET"]), ["text"]);
    assert_eq!(listed(&repo, &["--error", "E-NET"]).len(), 1);
}
