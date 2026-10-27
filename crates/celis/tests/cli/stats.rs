//! CIP-02, "Stats": the use of each live pattern, and the total.

use crate::support;

use support::{account, add, cli, err, finish, now, objects, out, stored_pattern, Repo, DAY};

/// The id `pattern <args>` prints.
fn decide(repo: &Repo, args: &[&str]) -> String {
    let mut all = vec!["pattern"];
    all.extend(args);
    let o = cli(repo, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().next().unwrap().to_string()
}

/// A done ask with a done step under it that uses `uses`; the step's id.
fn used(repo: &Repo, uses: &[&str], n: usize) -> String {
    let ask = add(repo, &["--", &format!("Ship part {n}")]);
    let mut args = vec!["--under", &ask[..8]];
    for id in uses {
        args.extend(["--uses", &id[..8]]);
    }
    let text = format!("build part {n}");
    args.extend(["--", &text]);
    let step = add(repo, &args);
    finish(repo, &step[..8]);
    finish(repo, &ask[..8]);
    step
}

/// `stats` prints each live pattern in scope, oldest first, with its uses, misled, replaces, last use,
/// examples and tidy reason, then the total: the index, the patterns written and replaced in the last
/// 7 days, and the fixes of done work whose trail used no pattern (`spec/02-cli.md`, "Stats").
#[test]
fn stats_prints_the_use_of_each_pattern_and_the_gaps() {
    let repo = Repo::new();
    let project = repo.project_id();
    let old = stored_pattern(
        &repo,
        (&project, "repo"),
        &account(),
        (None, None),
        "When releases ship: tag them.",
        now() - 61 * DAY,
    );
    let first = decide(&repo, &["--", "When prices round: round up."]);
    let kept = decide(
        &repo,
        &[
            "--replaces",
            &first[..8],
            "--",
            "When prices round: round down.",
        ],
    );
    std::thread::sleep(std::time::Duration::from_millis(1100));
    repo.write("flat.svg", "<svg/>\n");
    let misled = decide(
        &repo,
        &[
            "--good",
            "flat.svg",
            "--",
            "When logos render: keep them flat.",
        ],
    );
    used(&repo, &[&kept], 1);
    for n in [2, 3] {
        let step = used(&repo, &[&misled], n);
        add(
            &repo,
            &["--replaces", &step[..8], "--", &format!("Fix part {n}")],
        );
    }
    let plain = used(&repo, &[], 4);
    add(&repo, &["--replaces", &plain[..8], "--", "Fix part 4"]);
    let led = used(&repo, &[], 5);
    let fix = [
        "--replaces",
        &led[..8],
        "--uses",
        &kept[..8],
        "--",
        "Fix part 5",
    ];
    add(&repo, &fix);
    let text = out(&cli(&repo, &["stats"]));
    let json = objects(&repo, &["stats"]);
    let day = json[1]["pattern"]["used"].as_str().unwrap().to_string();
    assert_eq!(day.len(), 10, "{json:?}");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 4 + 17 + 1, "{text}");
    assert!(
        lines[4..21].iter().all(|l| l.starts_with("lifecycle ")),
        "{text}"
    );
    assert!(lines[21].starts_with("value last 28 days: asks:"), "{text}");
    assert_eq!(json[20]["lifecycle"]["span"], "month", "{json:?}");
    assert_eq!(json[21]["value"]["covered"], 3, "{json:?}");
    assert_eq!(
        lines[..4],
        [
            format!(
                "{} uses:0 misled:0 replaces:0 used:- examples:0 tidy:unused When releases ship",
                &old.id[..8]
            ),
            format!(
                "{} uses:2 misled:0 replaces:1 used:{day} examples:0 tidy:- When prices round",
                &kept[..8]
            ),
            format!(
                "{} uses:2 misled:2 replaces:0 used:{day} examples:1 tidy:misled When logos render",
                &misled[..8]
            ),
            "total 3 patterns in the index · last 7 days: 3 written, 1 replaced, 3 gaps"
                .to_string(),
        ],
        "{text}"
    );
    assert_eq!(
        json[0]["pattern"]["used"],
        serde_json::Value::Null,
        "{json:?}"
    );
    assert_eq!(json[0]["pattern"]["tidy"], serde_json::json!(["unused"]));
    assert_eq!(json[2]["pattern"]["misled"], 2, "{json:?}");
    assert_eq!(
        json[3]["total"],
        serde_json::json!({"index": 3, "written": 3, "replaced": 1, "gaps": 3})
    );
}
