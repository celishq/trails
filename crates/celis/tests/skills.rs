//! CIP-07, "Rules of a name" and "Overrides".

mod support;

use support::{add, cli, err, out, read_ids, Repo};

/// The trail of the skill `name` in the project `skill:<name>`, with a step that uses each skill of
/// `uses`, a rule on the trail saying `rule`, and the name; gives the trail's and the rule's ids.
fn skill(repo: &Repo, name: &str, uses: &[&str], rule: &str) -> (String, String) {
    let project = format!("skill:{name}");
    let trail = add(
        repo,
        &["--project", &project, "--", &format!("The skill {name}")],
    );
    for used in uses {
        let args = [
            "--project",
            &project,
            "--under",
            &trail[..8],
            "--uses",
            used,
        ];
        let mut args = args.to_vec();
        args.extend(["--", "Follow the skill it builds on"]);
        add(repo, &args);
    }
    let rule = decide(
        repo,
        &project,
        &format!("trail:{}", &trail[..8]),
        None,
        rule,
    );
    let o = cli(repo, &["name", &trail[..8], name]);
    assert!(o.status.success(), "{}", err(&o));
    (trail, rule)
}

/// The id `decide` prints for a rule of `project`, else of the main checkout's project, in `scope`,
/// replacing `replaces`.
fn decide(repo: &Repo, project: &str, scope: &str, replaces: Option<&str>, text: &str) -> String {
    let mut args = vec!["decide", "--scope", scope];
    if !project.is_empty() {
        args.extend(["--project", project]);
    }
    if let Some(id) = replaces {
        args.extend(["--replaces", id]);
    }
    args.extend(["--", text]);
    let o = cli(repo, &args);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().next().unwrap().to_string()
}

/// A step that uses a skill keeps the rules of the skill, and of each skill on its uses chain
/// (`spec/07-names.md`, "Rules of a name").
#[test]
fn a_step_keeps_the_rules_of_each_skill_on_the_chain() {
    let repo = Repo::new();
    let (_, base) = skill(
        &repo,
        "base",
        &[],
        "When you review: read the tests first, because they state the intent.",
    );
    let (_, layer) = skill(
        &repo,
        "layer",
        &["base"],
        "When you review: write no emoji, because the team asked.",
    );
    let kept = read_ids(&repo, &repo.path(), &["--uses", "layer"]);
    assert!(kept.contains(&layer[..8].to_string()), "{kept:?}");
    assert!(kept.contains(&base[..8].to_string()), "{kept:?}");
}

/// Two skills that use each other give each rule once (`spec/07-names.md`, "Rules of a name").
#[test]
fn skills_that_use_each_other_give_each_rule_once() {
    let repo = Repo::new();
    let (a, rule_a) = skill(&repo, "a", &[], "When a: do a, because a.");
    let (_, rule_b) = skill(&repo, "b", &["a"], "When b: do b, because b.");
    let back = ["--project", "skill:a", "--under", &a[..8], "--uses", "b"];
    let mut back = back.to_vec();
    back.extend(["--", "Follow b"]);
    add(&repo, &back);
    let kept = read_ids(&repo, &repo.path(), &["--uses", "a"]);
    assert_eq!(kept.len(), 2, "{kept:?}");
    assert!(kept.contains(&rule_a[..8].to_string()), "{kept:?}");
    assert!(kept.contains(&rule_b[..8].to_string()), "{kept:?}");
}
