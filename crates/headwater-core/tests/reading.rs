//! One reading of a checkout (31), which changes nothing in it (5).

use std::path::Path;
use std::process::Command;

use headwater_core::{read_repository, Pin};

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@example.invalid",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?}");
}

const LOCK: &str = r#"{"nodes": {
  "sluice": {"original": {"type": "github", "owner": "boomdrivedev", "repo": "sluice", "ref": "v0.97.2"}},
  "root": {"inputs": {"sluice": "sluice"}}}, "root": "root", "version": 7}"#;

/// The bytes of the checkout's files and of what git keeps of it: its
/// index, refs and configuration. The object store is left out, where git's
/// own background maintenance after the fixture's commit may write a lock.
fn snapshot(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(at) = stack.pop() {
        for entry in std::fs::read_dir(&at).unwrap() {
            let path = entry.unwrap().path();
            if path.ends_with(".git/objects") {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push((path.display().to_string(), std::fs::read(&path).unwrap()));
            }
        }
    }
    files.sort();
    files
}

#[test]
fn five_and_thirty_one_a_reading_names_its_commit_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    git(repo, &["init", "-q", "-b", "main"]);
    std::fs::write(repo.join("flake.lock"), LOCK).unwrap();
    std::fs::write(
        repo.join("flake.nix"),
        "{ inputs.sluice.url = \"github:boomdrivedev/sluice/v0.97.2\"; }\n",
    )
    .unwrap();
    git(repo, &["add", "flake.lock", "flake.nix"]);
    git(repo, &["commit", "-q", "-m", "fixture"]);
    // A change the commit does not hold, made after the index was written.
    std::fs::write(
        repo.join("flake.nix"),
        "{ inputs.sluice.url = \"github:boomdrivedev/sluice/v0.97.2\"; }\n\n",
    )
    .unwrap();

    let before = snapshot(repo);
    let reading = read_repository(repo);
    let after = snapshot(repo);
    let changed: Vec<&String> = after
        .iter()
        .filter(|file| !before.contains(file))
        .map(|(path, _)| path)
        .chain(
            before
                .iter()
                .filter(|file| !after.contains(file))
                .map(|(path, _)| path),
        )
        .collect();
    assert!(
        changed.is_empty(),
        "the reading wrote in the checkout: {changed:?}"
    );

    assert!(reading
        .state
        .commit
        .as_deref()
        .is_some_and(|c| c.len() == 40));
    assert!(reading.state.changes);
    assert_eq!(reading.descriptions, vec!["nix-flake".to_string()]);
    let sluice = &reading.deps.dependencies[0];
    assert_eq!(
        sluice.identity.to_string(),
        "pkg:github/boomdrivedev/sluice"
    );
    assert_eq!(sluice.pin(), Pin::Known("0.97.2".into()));
    assert_eq!(
        sluice.declarations[0].claims,
        vec!["flake.nix:1".to_string()]
    );
}

#[test]
fn c7_a_tiers_declaration_that_does_not_read_is_named_unread() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".config")).unwrap();
    std::fs::write(
        dir.path().join(".config/headwater.toml"),
        "[tiers.x]\nidentities = [\"sluice\"]\n",
    )
    .unwrap();
    let reading = read_repository(dir.path());
    assert_eq!(reading.deps.unread[0].location, ".config/headwater.toml");
    assert!(reading.descriptions.is_empty());
}
