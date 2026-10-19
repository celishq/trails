//! CIP-05, "Rules".

mod support;

use celis_nostr::{Account, Note};
use support::{account, cairn, drop_note, live_ids, now, out, put, rating, stored_rule, Repo, DAY};

fn short(id: &str) -> String {
    id[..8].to_string()
}

/// A reader counts a rule whatever account signed it (`spec/01-records.md`, "The account").
#[test]
fn a_rule_counts_whatever_account_signed_it() {
    let repo = Repo::plain();
    let id = repo.project_id();
    let project = (id.as_str(), "repo");
    let stranger = Account::new(&[2u8; 32]).unwrap();
    let signed = stored_rule(
        &repo,
        project,
        &account(),
        (None, None),
        "mine",
        now() - 3 * DAY,
    );
    let theirs = stored_rule(
        &repo,
        project,
        &stranger,
        (None, None),
        "theirs",
        now() - 2 * DAY,
    );
    assert_eq!(
        live_ids(&repo, &repo.path(), "repo"),
        [short(&theirs.id), short(&signed.id)]
    );
}

/// A rule is live until another rule names it in `replaces` (`spec/05-rules.md`,
/// "Rules").
#[test]
fn a_rule_is_live_until_another_rule_replaces_it() {
    let repo = Repo::plain();
    let id = repo.project_id();
    let project = (id.as_str(), "repo");
    let stranger = Account::new(&[2u8; 32]).unwrap();
    let old = stored_rule(
        &repo,
        project,
        &account(),
        (None, None),
        "old",
        now() - 4 * DAY,
    );
    let kept = stored_rule(
        &repo,
        project,
        &account(),
        (None, None),
        "kept",
        now() - 3 * DAY,
    );
    let gone = stored_rule(
        &repo,
        project,
        &stranger,
        (None, Some(&kept.id)),
        "no",
        now() - 2 * DAY,
    );
    let new = stored_rule(
        &repo,
        project,
        &account(),
        (None, Some(&old.id)),
        "new",
        now() - DAY,
    );
    assert_eq!(
        live_ids(&repo, &repo.path(), "repo"),
        [short(&new.id), short(&gone.id)]
    );
}

/// A rule is live until a drop names it (`spec/05-rules.md`, "Rules").
#[test]
fn a_rule_is_live_until_a_drop_names_it() {
    let repo = Repo::plain();
    let id = repo.project_id();
    let project = (id.as_str(), "repo");
    let kept = stored_rule(
        &repo,
        project,
        &account(),
        (None, None),
        "kept",
        now() - 3 * DAY,
    );
    let dropped = stored_rule(
        &repo,
        project,
        &account(),
        (None, None),
        "dropped",
        now() - 2 * DAY,
    );
    drop_note(&repo, &account(), &dropped.id, now() - DAY);
    assert_eq!(live_ids(&repo, &repo.path(), "repo"), [short(&kept.id)]);
    let o = repo.run(&repo.path(), &["grep", &short(&dropped.id)], &[], "");
    let text = out(&o);
    let rule = format!("rule {} dropped {{project}} dropped", short(&dropped.id));
    assert!(text.contains(&rule), "{text}");
    let reason = format!("dropped {} no longer holds", short(&dropped.id));
    assert!(text.contains(&reason), "{text}");
}

/// A reader ignores cairns, ratings, outcomes, source records and splits a store holds from before
/// (`spec/01-records.md`, "Records").
#[test]
fn a_reader_ignores_cairns_ratings_outcomes_source_records_and_splits_from_before() {
    let repo = Repo::plain();
    let id = repo.project_id();
    let project = (id.as_str(), "repo");
    let by = &account();
    let kept = stored_rule(
        &repo,
        project,
        &account(),
        (None, None),
        "kept",
        now() - 5 * DAY,
    );
    let old = cairn(&repo, by, "finding", &[("at", "a.rs:1")], now() - 4 * DAY);
    rating(&repo, by, &old, true, now() - 3 * DAY);
    let outcome = vec![
        vec!["I".to_string(), format!("project:{id}")],
        vec!["L".into(), "celis.outcome".into()],
        vec!["l".into(), "outcome".into(), "celis.outcome".into()],
        vec!["at".into(), "a.rs:1".into()],
    ];
    let note = Note::write(by, 1111u16, now() - 2 * DAY, outcome, "outcome").unwrap();
    put(&repo, &note);
    let source = vec![
        vec!["L".to_string(), "celis.source".into()],
        vec!["l".into(), "pdd".into(), "celis.source".into()],
        vec!["i".into(), format!("project:{id}")],
        vec!["decision".into(), "from a source record".into()],
        vec!["at".into(), "a.rs:1".into()],
    ];
    put(
        &repo,
        &Note::write(&account(), 1985u16, now() - DAY, source, "").unwrap(),
    );
    let split = vec![
        vec!["L".to_string(), "celis.step".into()],
        vec!["l".into(), "split".into(), "celis.step".into()],
        vec!["e".into(), note.id.clone()],
        vec!["i".into(), format!("project:{id}")],
        vec!["decision".into(), "from a split".into()],
        vec!["at".into(), "a.rs:1".into()],
    ];
    put(
        &repo,
        &Note::write(by, 1985u16, now() - DAY + 1, split, "").unwrap(),
    );
    assert_eq!(live_ids(&repo, &repo.path(), "repo"), [short(&kept.id)]);
}
