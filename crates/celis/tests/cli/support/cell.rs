//! The cells and the cell folders of another account, for the cell tests.

use super::{cli, err, now, out, put, Repo};
use celis_nostr::{Account, Note};

pub fn decide(repo: &Repo, text: &str) -> String {
    let o = cli(repo, &["pattern", "--", text]);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).lines().next().unwrap().to_string()
}

/// Another account, whose patterns and cells a test puts in the store.
pub fn theirs() -> Account {
    Account::new(&[7u8; 32]).unwrap()
}

/// A membership of `by` that puts `ids` in the cell `name`, put in the store.
pub fn membership(repo: &Repo, by: &Account, name: &str, ids: &[&str]) -> Note {
    let mut tags = vec![
        vec!["L".into(), "celis.cell".into()],
        vec!["l".into(), name.into(), "celis.cell".into()],
    ];
    tags.extend(ids.iter().map(|id| vec!["e".to_string(), id.to_string()]));
    let note = Note::write(by, 1985u16, now(), tags, "").unwrap();
    put(repo, &note);
    note
}

/// The number of record files in the store.
pub fn records(repo: &Repo) -> usize {
    std::fs::read_dir(repo.store().join("shared")).map_or(0, Iterator::count)
}

/// A folder that `theirs` exported: one pattern on another project, with a file example, and the
/// membership of `name` that names it.
pub fn their_folder(repo: &Repo, name: &str) -> (std::path::PathBuf, Note) {
    let folder = repo.path().join("from-them");
    std::fs::create_dir_all(folder.join("blobs")).unwrap();
    let sha = {
        use sha2::Digest;
        format!("{:x}", sha2::Sha256::digest(b"a bowl"))
    };
    let pubkey = theirs().pubkey().hex();
    let mut tags = super::pattern_tags(("q", "kitchen"));
    tags.push(vec![
        "good".into(),
        format!("blossom:{sha}.png?as={pubkey}&sz=6"),
        "bowl.png".into(),
    ]);
    let text = "Name: Rice first\nWhen a meal of rice is set\nDo: serve the rice first";
    let pattern = Note::write(&theirs(), 1111u16, now() - 10, tags, text).unwrap();
    let tags = vec![
        vec!["L".to_string(), "celis.cell".to_string()],
        vec!["l".to_string(), name.to_string(), "celis.cell".to_string()],
        vec!["e".to_string(), pattern.id.clone()],
    ];
    let membership = Note::write(&theirs(), 1985u16, now() - 5, tags, "").unwrap();
    let lines = [&pattern, &membership].map(|n| serde_json::to_string(n).unwrap());
    std::fs::write(folder.join("cell.jsonl"), lines.join("\n") + "\n").unwrap();
    std::fs::write(folder.join(format!("blobs/{sha}.png")), "a bowl").unwrap();
    std::fs::write(folder.join("cell.md"), "# east\n").unwrap();
    (folder, pattern)
}

/// Serves the files of `folder` over HTTP on a local port until the test ends; its base URL.
pub fn serve(folder: std::path::PathBuf) -> String {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut request = [0u8; 4096];
            let read = stream.read(&mut request).unwrap_or(0);
            let request = String::from_utf8_lossy(&request[..read]);
            let path = request
                .split(' ')
                .nth(1)
                .unwrap_or("/")
                .trim_start_matches('/');
            let reply = match std::fs::read(folder.join(path)) {
                Ok(body) => [
                    format!("HTTP/1.0 200 OK\r\nContent-Length: {}\r\n\r\n", body.len())
                        .into_bytes(),
                    body,
                ]
                .concat(),
                Err(_) => b"HTTP/1.0 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_vec(),
            };
            let _ = stream.write_all(&reply);
        }
    });
    format!("http://127.0.0.1:{port}")
}
