//! One parsed command line: its group and its name.

use crate::failure::Failure;
use crate::help::Group;
use crate::home::Home;
use celis_vault::{Announced, Keys, Store, Sync, Vaults};
#[cfg(feature = "vault")]
use celis_vault::{Blobs, Deleted, History, Moment, Scan, Vault, Version, Watch, When};
use celis_wire::{Account, Identity};
#[cfg(feature = "vault")]
use data_encoding::BASE64;
use serde_json::json;
#[cfg(feature = "vault")]
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// What a command line asked for.
pub struct Command {
    pub group: String,
    pub name: String,
    #[cfg(feature = "vault")]
    pub words: Vec<String>,
    pub key: Option<String>,
    pub phrase: Option<String>,
    pub json: bool,
    #[cfg(feature = "vault")]
    pub once: bool,
    pub yes: bool,
}

const TABLE: &[(&str, &str, &[&str])] = &[
    ("auth", "login", &["--key", "--phrase"]),
    ("auth", "status", &[]),
    ("auth", "logout", &[]),
    ("auth", "delete", &["--yes"]),
    #[cfg(feature = "vault")]
    ("vault", "create", &[]),
    #[cfg(feature = "vault")]
    ("vault", "clone", &[]),
    #[cfg(feature = "vault")]
    ("vault", "list", &[]),
    #[cfg(feature = "vault")]
    ("vault", "status", &[]),
    #[cfg(feature = "vault")]
    ("vault", "rename", &[]),
    #[cfg(feature = "vault")]
    ("vault", "remove", &[]),
    #[cfg(feature = "vault")]
    ("vault", "delete", &["--yes"]),
    #[cfg(feature = "vault")]
    ("file", "log", &[]),
    #[cfg(feature = "vault")]
    ("file", "show", &[]),
    #[cfg(feature = "vault")]
    ("file", "restore", &[]),
    #[cfg(feature = "vault")]
    ("sync", "run", &["--once"]),
];

impl Command {
    /// Reads the arguments after the program name; an unknown group, command or flag is a usage failure.
    pub fn parse(args: &[String]) -> Result<Command, Failure> {
        let mut words = args.iter().filter(|a| *a != "--json");
        let (Some(group), Some(name)) = (words.next(), words.next()) else {
            return Err(match args.iter().find(|a| *a != "--json") {
                Some(group) if Group::named(group).is_none() => Failure::unknown(&[group]),
                Some(group) => Failure::usage(
                    "expected: celis <group> <command>",
                    format!("celis help {group}"),
                ),
                None => Failure::usage("expected: celis <group> <command>", "celis help"),
            });
        };
        let Some((_, _, flags)) = TABLE.iter().find(|(g, n, _)| g == group && n == name) else {
            return Err(Failure::unknown(&[group, name]));
        };
        let mut positional = Vec::new();
        let mut key = None;
        let mut phrase = None;
        while let Some(word) = words.next() {
            if word.starts_with("--") {
                if !flags.contains(&word.as_str()) {
                    return Err(Failure::usage(
                        format!("unknown flag: {word}"),
                        format!("celis {group} {name} --help"),
                    ));
                }
                if word == "--key" {
                    key = Some(
                        words
                            .next()
                            .ok_or_else(|| {
                                Failure::usage(
                                    "--key needs an nsec",
                                    format!("celis {group} {name} --help"),
                                )
                            })?
                            .clone(),
                    );
                }
                if word == "--phrase" {
                    phrase = Some(
                        words
                            .next()
                            .ok_or_else(|| {
                                Failure::usage(
                                    "--phrase needs the words",
                                    format!("celis {group} {name} --help"),
                                )
                            })?
                            .clone(),
                    );
                }
            } else {
                positional.push(word.clone());
            }
        }
        Ok(Command {
            group: group.clone(),
            name: name.clone(),
            #[cfg(feature = "vault")]
            words: positional,
            key,
            phrase,
            json: args.iter().any(|a| a == "--json"),
            #[cfg(feature = "vault")]
            once: args.iter().any(|a| a == "--once"),
            yes: args.iter().any(|a| a == "--yes"),
        })
    }

    /// Runs the command and returns the bytes for stdout.
    pub fn run(&self) -> Result<Vec<u8>, Failure> {
        let destroys = matches!(
            (self.group.as_str(), self.name.as_str()),
            ("vault", "delete") | ("auth", "delete")
        );
        if destroys && !self.yes {
            return Err(self.usage(format!("{} {} needs --yes", self.group, self.name)));
        }
        let home = Home::new().map_err(Failure::error)?;
        match (self.group.as_str(), self.name.as_str()) {
            ("auth", "login") => self.login(&home).map(text),
            ("auth", "status") => Ok(text(self.npub(&account(&home)?))),
            ("auth", "logout") => self.logout(&home).map(text),
            ("auth", "delete") => self.close(&home).map(text),
            #[cfg(feature = "vault")]
            ("vault", "create") => self.create(&home).map(text),
            #[cfg(feature = "vault")]
            ("vault", "clone") => self.copy(&home).map(text),
            #[cfg(feature = "vault")]
            ("vault", "list") => self.list(&home).map(text),
            #[cfg(feature = "vault")]
            ("vault", "status") => self.status(&home).map(text),
            #[cfg(feature = "vault")]
            ("vault", "rename") => self.rename(&home).map(text),
            #[cfg(feature = "vault")]
            ("vault", "remove") => self.remove(&home).map(text),
            #[cfg(feature = "vault")]
            ("vault", "delete") => self.delete(&home).map(text),
            #[cfg(feature = "vault")]
            ("file", "log") => self.log(&home).map(text),
            #[cfg(feature = "vault")]
            ("file", "show") => self.show(&home),
            #[cfg(feature = "vault")]
            ("file", "restore") => self.restore(&home).map(text),
            #[cfg(feature = "vault")]
            ("sync", "run") => self.serve(&home).map(text),
            (group, name) => Err(Failure::unknown(&[group, name])),
        }
    }

    fn usage(&self, message: impl Into<String>) -> Failure {
        Failure::usage(
            message,
            format!("celis {} {} --help", self.group, self.name),
        )
    }

    /// Keeps the `--key` or `--phrase` account, or the one this host has, or a new one; only a new one
    /// prints its phrase, after the npub.
    fn login(&self, home: &Home) -> Result<Vec<String>, Failure> {
        let (account, new) = match (&self.key, &self.phrase) {
            (Some(_), Some(_)) => {
                return Err(self.usage("--key and --phrase cannot be used together"))
            }
            (Some(nsec), None) => (
                Account::parse_nsec(nsec)
                    .map_err(|e| self.usage(format!("--key is not an nsec: {e}")))?,
                None,
            ),
            (None, Some(phrase)) => (
                Identity::from_phrase(phrase)
                    .map_err(|e| self.usage(format!("--phrase: {e}")))?
                    .account()
                    .clone(),
                None,
            ),
            (None, None) => match home.account().map_err(Failure::error)? {
                Some(account) => (account, None),
                None => {
                    let identity = Identity::generate().map_err(Failure::error)?;
                    (identity.account().clone(), Some(identity.phrase()))
                }
            },
        };
        home.keep(&account).map_err(Failure::error)?;
        let _ = Store::new(&relay(), account.clone(), account.clone()).create();
        let Some(phrase) = new else {
            return Ok(self.npub(&account));
        };
        let npub = account.pubkey().npub();
        Ok(if self.json {
            vec![json!({ "npub": npub, "phrase": phrase }).to_string()]
        } else {
            vec![npub, phrase]
        })
    }

    /// Forgets the account on this host.
    fn logout(&self, home: &Home) -> Result<Vec<String>, Failure> {
        home.forget().map_err(Failure::error)?;
        Ok(if self.json {
            vec![json!({"logged_out": true}).to_string()]
        } else {
            Vec::new()
        })
    }

    /// Deletes every vault the account owns, everywhere, then its store, then forgets it here.
    fn close(&self, home: &Home) -> Result<Vec<String>, Failure> {
        let account = account(home)?;
        let vaults = Vaults::fetch(&relay(), &account).map_err(unreachable)?;
        for vault in vaults.all() {
            erase(&account, vault)?;
        }
        Store::new(&relay(), account.clone(), account.clone())
            .delete()
            .map_err(unreachable)?;
        home.forget().map_err(Failure::error)?;
        Ok(if self.json {
            vec![json!({"deleted": true}).to_string()]
        } else {
            Vec::new()
        })
    }

    /// Makes the copy at the working folder a plain folder again.
    #[cfg(feature = "vault")]
    fn remove(&self, home: &Home) -> Result<Vec<String>, Failure> {
        let vault = open(true)?;
        let folder = vault.root().to_path_buf();
        home.remove(&folder).map_err(Failure::error)?;
        vault.remove().map_err(Failure::error)?;
        let folder = folder.display().to_string();
        Ok(if self.json {
            vec![json!({"removed": folder}).to_string()]
        } else {
            Vec::new()
        })
    }

    /// Deletes the vault the first word names, by id or name, on the relay and Blossom.
    #[cfg(feature = "vault")]
    fn delete(&self, home: &Home) -> Result<Vec<String>, Failure> {
        let account = account(home)?;
        let word = self
            .words
            .first()
            .ok_or_else(|| self.usage("expected: celis vault delete <name>"))?;
        let vaults = Vaults::fetch(&relay(), &account).map_err(unreachable)?;
        let name = match vaults.find(word) {
            Some(found) => {
                erase(&account, found)?;
                found.name.clone()
            }
            None => vaults
                .find_deleted(word)
                .map(|gone| gone.name.clone())
                .ok_or_else(|| Failure::named("NotFound", format!("no vault is named {word}")))?,
        };
        Ok(vec![if self.json {
            json!({"deleted": name}).to_string()
        } else {
            name
        }])
    }

    /// Makes the working folder a vault named by the first word or after the folder, lists it here, and
    /// announces it on the relay, or leaves that to the next sync.
    #[cfg(feature = "vault")]
    fn create(&self, home: &Home) -> Result<Vec<String>, Failure> {
        let account = account(home)?;
        let folder = std::env::current_dir().map_err(|e| Failure::error(e.into()))?;
        let name = match self.words.first() {
            Some(name) => name.clone(),
            None => folder
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
        };
        let (relay, blossom) = (relay(), blossom());
        let vault = Vault::create(&folder, &name, &relay, &blossom).map_err(Failure::error)?;
        let keys = vault.keys(&account).map_err(Failure::error)?;
        let _ = Sync::new(&relay, &blossom, keys).announce(&vault);
        home.add(vault.root()).map_err(Failure::error)?;
        let folder = vault.root().display().to_string();
        Ok(vec![if self.json {
            json!({"name": name, "folder": folder}).to_string()
        } else {
            format!("{name} {folder}")
        }])
    }

    /// Makes a copy of the vault the first word names, by id or name, in the folder the second word names or
    /// one after the vault, lists it here, and syncs it once, or leaves what failed to the next sync.
    #[cfg(feature = "vault")]
    fn copy(&self, home: &Home) -> Result<Vec<String>, Failure> {
        let account = account(home)?;
        let word = self
            .words
            .first()
            .ok_or_else(|| self.usage("expected: celis vault clone <name> [<folder>]"))?;
        let vaults = Vaults::fetch(&relay(), &account).map_err(unreachable)?;
        let found = vaults
            .find(word)
            .ok_or_else(|| Failure::named("NotFound", format!("no vault is named {word}")))?;
        let here = std::env::current_dir().map_err(|e| Failure::error(e.into()))?;
        let folder = here.join(self.words.get(1).unwrap_or(&found.name));
        let mut vault = Vault::copy(
            &folder,
            &found.id,
            &found.name,
            &found.relay,
            &found.blossom,
        )
        .map_err(Failure::error)?;
        home.add(vault.root()).map_err(Failure::error)?;
        let keys = vault.keys(&account).map_err(Failure::error)?;
        let _ = Sync::new(&found.relay, &found.blossom, keys).once(&mut vault);
        let folder = vault.root().display().to_string();
        Ok(vec![if self.json {
            json!({"name": found.name, "folder": folder}).to_string()
        } else {
            format!("{} {folder}", found.name)
        }])
    }

    /// Every vault the account's store names and every one listed here, each with its copies here and
    /// whether each is in sync.
    #[cfg(feature = "vault")]
    fn list(&self, home: &Home) -> Result<Vec<String>, Failure> {
        let account = account(home)?;
        let vaults = Vaults::fetch(&relay(), &account).map_err(unreachable)?;
        let mut named: Vec<(String, String)> = vaults
            .all()
            .iter()
            .map(|a| (a.id.clone(), a.name.clone()))
            .collect();
        let mut copies = Vec::new();
        for folder in home.copies().map_err(Failure::error)? {
            let vault = Vault::open(&folder, false).map_err(Failure::error)?;
            let (waiting, changed, skipped) = state(&vault, &account)?;
            let in_sync = waiting == 0 && changed.is_empty() && skipped.is_empty();
            if vaults.deleted(vault.id()) {
                continue;
            }
            if !named.iter().any(|(id, _)| id == vault.id()) {
                named.push((vault.id().to_string(), vault.name().to_string()));
            }
            copies.push((vault.id().to_string(), folder, in_sync));
        }
        let mut lines = Vec::new();
        for (id, name) in named {
            let here: Vec<_> = copies.iter().filter(|(of, _, _)| *of == id).collect();
            if self.json {
                let here: Vec<_> = here
                    .iter()
                    .map(|(_, folder, in_sync)| json!({"folder": folder, "in_sync": in_sync}))
                    .collect();
                lines.push(json!({"id": id, "name": name, "copies": here}).to_string());
                continue;
            }
            lines.push(name);
            for (_, folder, in_sync) in here {
                let said = if *in_sync { "in sync" } else { "not in sync" };
                lines.push(format!("  {} {said}", folder.display()));
            }
        }
        Ok(lines)
    }

    /// Names this copy's vault after the first word, and publishes the name for every copy, or leaves that
    /// to the next sync.
    #[cfg(feature = "vault")]
    fn rename(&self, home: &Home) -> Result<Vec<String>, Failure> {
        let account = account(home)?;
        let name = self
            .words
            .first()
            .ok_or_else(|| self.usage("expected: celis vault rename <name>"))?;
        let mut vault = open(true)?;
        vault.rename(name).map_err(Failure::error)?;
        let keys = vault.keys(&account).map_err(Failure::error)?;
        let _ = Sync::new(vault.relay(), vault.blossom(), keys).announce(&vault);
        Ok(vec![if self.json {
            json!({ "name": name }).to_string()
        } else {
            name.clone()
        }])
    }

    /// Syncs every copy in the host's list, holding each one's lock: once with `--once`, else until stopped.
    /// A file is recorded once it settles and published at once; a copy syncs when its store hears a wrap,
    /// and every minute besides. With `--json` it prints the counts of each pass that syncs a copy.
    #[cfg(feature = "vault")]
    fn serve(&self, home: &Home) -> Result<Vec<String>, Failure> {
        let account = account(home)?;
        let mut copies = Vec::new();
        for folder in home.copies().map_err(Failure::error)? {
            let vault = Vault::open(&folder, true).map_err(Failure::error)?;
            let keys = vault.keys(&account).map_err(Failure::error)?;
            let sync = Sync::new(vault.relay(), vault.blossom(), keys.clone());
            let watch = Watch::new(vault.root().to_path_buf());
            copies.push((vault, keys, sync, watch));
        }
        if self.once {
            let mut failed = None;
            let mut deleted = None;
            let mut pass = Pass::default();
            for (vault, keys, sync, _) in &mut copies {
                let done = Scan::new(None)
                    .run(vault, keys)
                    .and_then(|_| sync.exchange(vault));
                pass.count(&done);
                if let Err(error) = done {
                    match forget_deleted(home, vault, &error) {
                        Some(failure) => {
                            deleted.get_or_insert(failure);
                        }
                        None => {
                            failed.get_or_insert(error);
                        }
                    }
                }
            }
            if self.json && pass.copies > 0 {
                println!("{}", pass.line());
            }
            if let Some(failure) = deleted {
                return Err(failure);
            }
            return failed.map_or(Ok(Vec::new()), |error| Err(unreachable(error)));
        }
        let stores: Vec<_> = copies
            .iter()
            .map(|(_, _, sync, _)| sync.follow().ok())
            .collect();
        let mut synced: Option<Instant> = None;
        let mut gone: Vec<std::path::PathBuf> = Vec::new();
        loop {
            let due = synced.is_none_or(|at| at.elapsed() >= PERIOD);
            let mut pass = Pass::default();
            for ((vault, keys, sync, watch), store) in copies.iter_mut().zip(&stores) {
                if gone.iter().any(|root| root == vault.root()) {
                    continue;
                }
                let settled = watch.settled();
                let mut recorded = false;
                if !settled.is_empty() {
                    if let Ok(scanned) = Scan::new(Some(settled)).run(vault, keys) {
                        let _ = watch.mark();
                        recorded = scanned.recorded > 0;
                    }
                }
                let heard = store.as_ref().is_some_and(|s| s.heard());
                let done = if due || heard {
                    sync.exchange(vault)
                } else if recorded {
                    sync.publish(vault).map(|sent| (sent, 0))
                } else {
                    continue;
                };
                pass.count(&done);
                if let Err(error) = done {
                    match forget_deleted(home, vault, &error) {
                        Some(failure) => {
                            eprintln!("{}", failure.render(self.json));
                            gone.push(vault.root().to_path_buf());
                        }
                        None => eprintln!("{}", unreachable(error).render(self.json)),
                    }
                }
            }
            if self.json && pass.copies > 0 {
                println!("{}", pass.line());
            }
            if due {
                synced = Some(Instant::now());
            }
            let next = copies.iter().filter_map(|(_, _, _, w)| w.next()).min();
            let wait = next.map_or(POLL, |at| at.saturating_duration_since(Instant::now()));
            std::thread::sleep(wait.min(POLL));
        }
    }

    /// This copy: whether it is in sync, how many records wait to be sent, what the next scan would record,
    /// what the last scan skipped, and which files stand at a merge.
    #[cfg(feature = "vault")]
    fn status(&self, home: &Home) -> Result<Vec<String>, Failure> {
        let account = account(home)?;
        let vault = open(false)?;
        let keys = vault.keys(&account).map_err(Failure::error)?;
        let (waiting, changed, skipped) = state(&vault, &account)?;
        let merged: Vec<String> = vault
            .files(&keys)
            .map_err(Failure::error)?
            .iter()
            .filter(|v| v.entry.source == "merge")
            .map(|v| v.entry.path.clone())
            .collect();
        let in_sync = waiting == 0 && changed.is_empty() && skipped.is_empty();
        if self.json {
            let line = json!({
                "in_sync": in_sync, "waiting": waiting, "changed": changed,
                "skipped": skipped, "merged": merged,
            });
            return Ok(vec![line.to_string()]);
        }
        let mut lines = vec![if in_sync { "in sync" } else { "not in sync" }.to_string()];
        lines.push(format!("waiting {waiting}"));
        lines.extend(changed.iter().map(|p| format!("changed {p}")));
        lines.extend(skipped.iter().map(|p| format!("skipped {p}")));
        lines.extend(merged.iter().map(|p| format!("merged {p}")));
        Ok(lines)
    }

    /// The versions of the file at the path in the first word, or of the vault, newest first.
    #[cfg(feature = "vault")]
    fn log(&self, home: &Home) -> Result<Vec<String>, Failure> {
        let account = account(home)?;
        let vault = open(false)?;
        let keys = vault.keys(&account).map_err(Failure::error)?;
        let lines = match self.words.first() {
            Some(path) => vault
                .versions(&keys, &within(&vault, path)?)
                .map_err(Failure::error)?
                .iter()
                .map(|v| self.line(v))
                .collect(),
            None => vault
                .history(&keys)
                .map_err(Failure::error)?
                .all()
                .iter()
                .map(|v| self.line(v))
                .collect(),
        };
        Ok(lines)
    }

    /// The bytes of the version the first word names, exactly.
    #[cfg(feature = "vault")]
    fn show(&self, home: &Home) -> Result<Vec<u8>, Failure> {
        let account = account(home)?;
        let vault = open(false)?;
        let keys = vault.keys(&account).map_err(Failure::error)?;
        let (path, version) = self.named(&vault, &keys)?;
        ensure_kept(&version, &path)?;
        let objects = vault.root().join(".celis/objects");
        let bytes = Blobs::new(vault.blossom(), &keys, &objects)
            .and_then(|blobs| blobs.bytes(&version.entry))
            .map_err(Failure::error)?;
        if self.json {
            let id = version.entry.id();
            let line = json!({"id": id, "path": path, "size": bytes.len(), "bytes": BASE64.encode(&bytes)});
            return Ok(text(vec![line.to_string()]));
        }
        Ok(bytes)
    }

    /// Puts the version the first word names back, as a new version.
    #[cfg(feature = "vault")]
    fn restore(&self, home: &Home) -> Result<Vec<String>, Failure> {
        let account = account(home)?;
        let mut vault = open(true)?;
        let keys = vault.keys(&account).map_err(Failure::error)?;
        let (path, version) = self.named(&vault, &keys)?;
        ensure_kept(&version, &path)?;
        let id = vault
            .restore(&keys, &version, &path)
            .map_err(Failure::error)?;
        Ok(vec![if self.json {
            json!({"id": id, "path": path}).to_string()
        } else {
            format!("{} {path}", &id[..8])
        }])
    }

    /// The path and the one version of its file that the first word, `<path>@<when>`, names.
    #[cfg(feature = "vault")]
    fn named(&self, vault: &Vault, keys: &Keys) -> Result<(String, Version), Failure> {
        let word = self.words.first().map(String::as_str).unwrap_or_default();
        let (path, when) = word
            .rsplit_once('@')
            .ok_or_else(|| self.usage("expected <path>@<when>"))?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let when = When::parse(when, now).map_err(|e| self.usage(format!("{e}")))?;
        let path = within(vault, path)?;
        let mut versions = vault.versions(keys, &path).map_err(Failure::error)?;
        versions.reverse();
        let history = History::of(versions);
        let mut found = history.named(&path, &when);
        match (found.pop(), found.is_empty()) {
            (Some(version), true) => Ok((path, version.clone())),
            (Some(_), false) => Err(Failure::named(
                "Ambiguous",
                format!("more than one version of {path} starts that way"),
            )),
            (None, _) => Err(Failure::named(
                "NotFound",
                format!("no version of {path} at that time"),
            )),
        }
    }

    /// One version as a line: its fields under `--json`, else its short id, time, op and path.
    #[cfg(feature = "vault")]
    fn line(&self, version: &Version) -> String {
        let entry = &version.entry;
        let at = Moment(version.at).to_string();
        if !self.json {
            return format!("{} {at} {} {}", &entry.id()[..8], entry.op, entry.path);
        }
        let mut line = json!({
            "id": entry.id(), "file": entry.file, "path": entry.path, "op": entry.op,
            "before": entry.before, "hash": entry.hash, "size": entry.size,
            "author": version.author, "source": entry.source, "at": at,
        });
        if let Some(from) = &entry.from {
            line["from"] = json!(from);
        }
        line.to_string()
    }

    fn npub(&self, account: &Account) -> Vec<String> {
        let npub = account.pubkey().npub();
        vec![if self.json {
            json!({ "npub": npub }).to_string()
        } else {
            npub
        }]
    }
}

#[cfg(feature = "vault")]
const PERIOD: Duration = Duration::from_secs(60);
#[cfg(feature = "vault")]
const POLL: Duration = Duration::from_millis(100);
const RELAY: &str = "wss://relay.celis.ai";
#[cfg(feature = "vault")]
const BLOSSOM: &str = "https://blossom.celis.ai";

/// The copies one pass of `sync run` synced, and the records they sent and received.
#[cfg(feature = "vault")]
#[derive(Default)]
struct Pass {
    copies: usize,
    sent: usize,
    received: usize,
}

#[cfg(feature = "vault")]
impl Pass {
    /// Adds one copy's sync to the pass, unless it failed.
    fn count<E>(&mut self, done: &Result<(usize, usize), E>) {
        if let Ok((sent, received)) = done {
            self.copies += 1;
            self.sent += sent;
            self.received += received;
        }
    }

    fn line(&self) -> String {
        json!({"copies": self.copies, "sent": self.sent, "received": self.received}).to_string()
    }
}

fn relay() -> String {
    std::env::var("CELIS_RELAY").unwrap_or_else(|_| RELAY.to_string())
}

#[cfg(feature = "vault")]
fn blossom() -> String {
    std::env::var("CELIS_BLOSSOM").unwrap_or_else(|_| BLOSSOM.to_string())
}

/// The account this host keeps, or a failure that says to log in first.
fn account(home: &Home) -> Result<Account, Failure> {
    home.account()
        .map_err(Failure::error)?
        .ok_or_else(|| Failure::named("NoAccount", "no account yet: run celis auth login"))
}

/// What `vault` has not settled: how many records wait to be sent, the paths the next scan would record,
/// and what the last scan skipped.
#[cfg(feature = "vault")]
fn state(vault: &Vault, account: &Account) -> Result<(usize, Vec<String>, Vec<String>), Failure> {
    let keys = || vault.keys(account).map_err(Failure::error);
    let waiting = Sync::new(vault.relay(), vault.blossom(), keys()?)
        .waiting(vault)
        .map_err(Failure::error)?;
    let changed = Scan::new(None)
        .differs(vault, &keys()?)
        .map_err(Failure::error)?;
    let skipped = Scan::skipped(vault)
        .iter()
        .map(|p| p.display().to_string())
        .collect();
    Ok((waiting, changed, skipped))
}

/// Deletes one announced vault on its relay and Blossom.
fn erase(account: &Account, vault: &Announced) -> Result<(), Failure> {
    let keys = Keys::new(account.clone(), &vault.id).map_err(Failure::error)?;
    Sync::new(&vault.relay, &vault.blossom, keys)
        .delete(vault)
        .map_err(unreachable)
}

/// Drops `vault` from the host's list when `error` says it was deleted, and returns that failure.
#[cfg(feature = "vault")]
fn forget_deleted(home: &Home, vault: &Vault, error: &anyhow::Error) -> Option<Failure> {
    error.downcast_ref::<Deleted>()?;
    let _ = home.remove(vault.root());
    Some(Failure::named("vault-deleted", format!("{error:#}")))
}

/// A sync that could not finish, most often because the relay or Blossom cannot be reached.
fn unreachable(error: anyhow::Error) -> Failure {
    Failure::named("Unreachable", format!("{error:#}"))
}

/// The nearest vault above the working folder.
#[cfg(feature = "vault")]
fn open(write: bool) -> Result<Vault, Failure> {
    let folder = std::env::current_dir().map_err(|e| Failure::error(e.into()))?;
    Vault::open(&folder, write).map_err(Failure::error)
}

/// `path`, named from the working folder, as a path inside `vault`.
#[cfg(feature = "vault")]
fn within(vault: &Vault, path: &str) -> Result<String, Failure> {
    let folder = std::env::current_dir().map_err(|e| Failure::error(e.into()))?;
    let full = folder.join(path);
    let inside = full.strip_prefix(vault.root()).unwrap_or(&full);
    Ok(inside.to_string_lossy().trim_matches('/').to_string())
}

/// A failure when `version` is a delete, which holds no bytes.
#[cfg(feature = "vault")]
fn ensure_kept(version: &Version, path: &str) -> Result<(), Failure> {
    if version.entry.op == "delete" {
        return Err(Failure::named(
            "NotFound",
            format!("{path} was deleted at that time"),
        ));
    }
    Ok(())
}

/// `lines` as the bytes of stdout, a newline after each.
fn text(lines: Vec<String>) -> Vec<u8> {
    lines
        .iter()
        .flat_map(|l| format!("{l}\n").into_bytes())
        .collect()
}
