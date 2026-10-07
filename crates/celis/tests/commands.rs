//! CIP-11, "Commands".

mod support;
use serde_json::Value;
use std::process::Output;
use support::{agent, cairn, cairn_saying, err, labelled, now, out, Repo, DAY};

fn write(repo: &Repo, env: &[(&str, &str)], args: &[&str]) -> Output {
    let mut all = vec!["add", "--task", "t"];
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
        let args = ["finding", "--", text];
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

/// `rate` refuses an id no cairn in the store has, with `no-cairn` (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn rate_refuses_an_unknown_id_with_no_cairn() {
    let repo = Repo::new();
    let id = "0".repeat(64);
    let o = repo.run(&repo.path(), &["rate", "--json", &id, "+"], &[], "");
    assert_eq!(o.status.code(), Some(1));
    let object: Value = serde_json::from_str(err(&o).trim()).unwrap();
    assert_eq!(object["error"], "no-cairn");
}

/// With `$CELIS_SESSION` set, `rate` is signed by that session's agent, not the account (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn with_a_session_rate_signs_as_that_sessions_agent() {
    let repo = Repo::new();
    let session = [("CELIS_SESSION", "s1")];
    let args = ["finding", "--", "one"];
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

    let notes: Vec<celis_wire::Note> = std::fs::read_dir(repo.store().join("shared"))
        .unwrap()
        .filter_map(|e| std::fs::read_to_string(e.unwrap().path()).ok())
        .filter_map(|t| serde_json::from_str(&t).ok())
        .collect();
    let rating = notes.iter().find(|n| n.kind == 7).unwrap();
    assert_eq!(rating.pubkey.npub(), agent_npub.as_str().unwrap());
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

/// `list` prints every cairn, in reading order, with no limit
/// (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn list_must_print_every_live_cairn_with_no_limit() {
    let repo = Repo::new();
    let by = agent(&repo, "a");
    let ids: Vec<String> = (0..25)
        .map(|i| cairn(&repo, &by, "finding", &[], now() - (30 - i) * 3600).id)
        .collect();
    let printed: Vec<String> = listed(&repo, &[])
        .iter()
        .filter_map(|e| Some(e["id"].as_str()?.to_string()))
        .collect();
    let newest_first: Vec<String> = ids.iter().rev().cloned().collect();
    assert_eq!(printed, newest_first);
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

/// `add` refuses a missing or empty `--task` as a usage error, and writes nothing
/// (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn add_must_refuse_a_missing_or_empty_task() {
    let repo = Repo::new();
    for args in [
        &["add", "--", "Before you run make, set CI=1."][..],
        &["add", "--task", " ", "--", "Before you run make, set CI=1."],
    ] {
        let o = repo.run(&repo.path(), args, &[], "");
        assert_eq!(o.status.code(), Some(2), "{args:?}: {}", err(&o));
    }
    assert!(labelled(&repo, "finding").is_empty());
}

/// `list` prints every cairn of every project, an unrated one of another project included, by score
/// (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn list_prints_every_cairn_of_every_project_by_score() {
    let repo = Repo::new();
    let shop = repo.other("shop");
    let mut ids = vec![];
    for text in ["Before you test, set A=1.", "Before you test, set B=1."] {
        let args = ["add", "--task", "t", "--", text];
        let o = repo.run(&shop, &args, &[], "");
        assert!(o.status.success(), "{}", err(&o));
        ids.push(out(&o).trim().to_string());
    }
    let here = write(&repo, &[], &["--", "Before you test, set C=1."]);
    assert!(here.status.success(), "{}", err(&here));
    let rated = repo.run(
        &shop,
        &["rate", &ids[0], "+"],
        &[("CELIS_SESSION", "s2")],
        "",
    );
    assert!(rated.status.success(), "{}", err(&rated));

    {
        let all = listed(&repo, &[]);
        let cairns: Vec<&Value> = all.iter().filter(|e| e.get("id").is_some()).collect();
        assert_eq!(cairns.len(), 3, "{all:?}");
        assert_eq!(cairns[0]["id"], ids[0].as_str());
        let scores: Vec<f64> = cairns
            .iter()
            .map(|e| e["score"].as_f64().unwrap())
            .collect();
        assert!(scores.windows(2).all(|w| w[0] >= w[1]), "{scores:?}");
    }
}

/// `list --prompt` prints what `list` prints, in the same order (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn list_prompt_must_print_what_list_prints() {
    let repo = Repo::new();
    let by = agent(&repo, "a");
    for (i, text) in ["first", "second", "third"].iter().enumerate() {
        cairn_saying(&repo, &by, text, now() - (10 - i as u64) * 3600);
    }
    let plain = repo.run(&repo.path(), &["list"], &[], "");
    let prompted = repo.run(
        &repo.path(),
        &["list", "--prompt", "fix the build"],
        &[],
        "",
    );
    assert!(plain.status.success(), "{}", err(&plain));
    assert!(prompted.status.success(), "{}", err(&prompted));
    assert_eq!(out(&prompted), out(&plain));
}

/// `add` refuses --weight 0, 11, 2.5 and x as a usage error and writes nothing
/// (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn add_refuses_a_weight_that_is_not_a_whole_number_1_to_10() {
    let repo = Repo::new();
    for weight in ["1", "10"] {
        let o = write(&repo, &[], &["--weight", weight, "--", weight]);
        assert!(o.status.success(), "{weight}: {}", err(&o));
    }
    for weight in ["0", "11", "2.5", "x"] {
        let o = write(&repo, &[], &["--weight", weight, "--", "text"]);
        assert_eq!(o.status.code(), Some(2), "{weight}: {}", err(&o));
    }
    assert_eq!(contents(&repo, &[]), ["1", "10"]);
}

/// A flag no command takes is a usage error, on a call that is valid without it
/// (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn a_flag_no_command_takes_is_a_usage_error() {
    let repo = Repo::new();
    let session = [("CELIS_SESSION", "s1")];
    for (call, env) in [
        (&["list"][..], &[][..]),
        (&["report", "--done"], &session[..]),
    ] {
        let o = repo.run(&repo.path(), call, env, "");
        assert!(o.status.success(), "{call:?}: {}", err(&o));
        for flag in [
            "--error",
            "--context",
            "--spot",
            "--else",
            "--helped",
            "--wrong",
        ] {
            let mut all = call.to_vec();
            all.extend([flag, "x"]);
            let o = repo.run(&repo.path(), &all, env, "");
            assert_eq!(o.status.code(), Some(2), "{all:?}: {}", err(&o));
        }
    }
}

/// `add --weight 7` writes the tag `["weight","7"]` (`spec/cairns/11-cli.md`, "Commands").
#[test]
fn add_weight_writes_the_weight_tag() {
    let repo = Repo::new();
    let o = write(&repo, &[], &["--weight", "7", "--", "heavy"]);
    assert!(o.status.success(), "{}", err(&o));
    let notes: Vec<celis_wire::Note> = std::fs::read_dir(repo.store().join("shared"))
        .unwrap()
        .filter_map(|e| std::fs::read_to_string(e.unwrap().path()).ok())
        .filter_map(|t| serde_json::from_str(&t).ok())
        .collect();
    let note = notes.iter().find(|n| n.content.contains("heavy")).unwrap();
    assert!(note
        .tags
        .contains(&vec!["weight".to_string(), "7".to_string()]));
}
