//! `celis setup`.

use crate::support;

use support::{cli, err, out, refusal, tiny_model, Repo};

/// Off a terminal with no `--model`, `setup` prints the list, the recommended model first.
#[test]
fn off_a_terminal_setup_prints_the_list() {
    let repo = Repo::new();
    let o = cli(&repo, &["setup"]);
    assert!(o.status.success(), "{}", err(&o));
    let ids: Vec<String> = out(&o)
        .lines()
        .map(|l| l.split(' ').next().unwrap().to_string())
        .collect();
    assert_eq!(ids, ["embeddinggemma", "e5-small", "none"]);
    assert!(out(&o).lines().next().unwrap().contains(" recommended "));
}

/// `setup` refuses a model that is not in the list.
#[test]
fn setup_refuses_a_model_not_in_the_list() {
    let repo = Repo::new();
    refusal(
        &cli(&repo, &["setup", "--json", "--model", "gpt"]),
        2,
        "usage",
    );
}

/// `none` turns search off and removes the downloaded models.
#[test]
fn none_turns_search_off() {
    let repo = Repo::new();
    tiny_model(&repo);
    let o = cli(&repo, &["setup", "--model", "none"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(!repo.home().join("models").exists());
    let o = cli(&repo, &["search", "--kind", "step", "--", "prices"]);
    assert!(
        out(&o).starts_with("celis search has no model"),
        "{}",
        out(&o)
    );
}

/// When a download or its sha256 check fails, `setup` keeps the model it had.
#[test]
fn a_failed_download_or_check_keeps_the_model_there_was() {
    let repo = Repo::new();
    tiny_model(&repo);
    let served = tempfile::tempdir().unwrap();
    let base = format!("file://{}", served.path().display());
    let env = [("CELIS_MODELS_BASE", base.as_str())];
    let o = repo.run(
        &repo.path(),
        &["setup", "--model", "embeddinggemma"],
        &env,
        "",
    );
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    let folder = served
        .path()
        .join("unsloth/embeddinggemma-300m/resolve/bfa3c846ac738e62aa61806ef9112d34acb1dc5a");
    std::fs::create_dir_all(&folder).unwrap();
    for file in [
        "config.json",
        "tokenizer.json",
        "model.safetensors",
        "2_Dense/model.safetensors",
        "3_Dense/model.safetensors",
    ] {
        std::fs::create_dir_all(folder.join(file).parent().unwrap()).unwrap();
        std::fs::write(folder.join(file), "not the model").unwrap();
    }
    let o = repo.run(
        &repo.path(),
        &["setup", "--model", "embeddinggemma"],
        &env,
        "",
    );
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    assert!(err(&o).contains("does not match its sha256"), "{}", err(&o));
    assert!(repo
        .home()
        .join("models/e5-small/model.safetensors")
        .exists());
    assert!(!repo.home().join("models/embeddinggemma").exists());
    let o = cli(&repo, &["search", "--", "rust"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(!out(&o).starts_with("celis search has no model"));
}
