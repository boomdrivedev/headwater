//! A scan from the command line's own code, over a checkout on disk and
//! recorded lookups: the binding gives each identity its lookup (51; C.3),
//! and the binary's capture of a repository no lookup is asked about.

use std::path::Path;
use std::process::Command;

use headwater::scan::{scan, Lookups};
use headwater_core::report::Status;
use headwater_core::Options;
use headwater_lookups::github::GraphQl;
use headwater_lookups::timber::StatusSource;
use serde_json::{json, Value};

const LOCK: &str = r#"{"nodes": {
  "sluice": {"original": {"type": "github", "owner": "boomdrivedev", "repo": "sluice", "ref": "v0.21.2"}},
  "worktrunk": {"original": {"type": "github", "owner": "max-sixty", "repo": "worktrunk", "ref": "v0.80.0"}},
  "root": {"inputs": {"sluice": "sluice", "worktrunk": "worktrunk"}}}, "root": "root", "version": 7}"#;

const CONFIG: &str = r#"
[tiers.family]
prefixes = ["pkg:github/boomdrivedev/"]

[[timber]]
identities = ["pkg:github/boomdrivedev/sluice"]
channel = "stable"
"#;

fn checkout(dir: &Path, lock: &str, config: Option<&str>) {
    std::fs::write(dir.join("flake.lock"), lock).unwrap();
    if let Some(config) = config {
        std::fs::create_dir_all(dir.join(".config")).unwrap();
        std::fs::write(dir.join(".config/headwater.toml"), config).unwrap();
    }
}

/// GitHub's answer for worktrunk alone; a query naming sluice fails the test.
struct GitHub;

impl GraphQl for GitHub {
    fn query(&self, query: &str) -> Result<Value, String> {
        assert!(
            !query.contains("sluice"),
            "GitHub was asked of an identity timber answers for (C.3)"
        );
        assert!(query.contains("name: \"worktrunk\""));
        Ok(
            json!({"data": {"r0": {"releases": {"pageInfo": {"hasNextPage": false, "endCursor": null}, "nodes": [
                {"tagName": "v0.82.0", "url": "https://github.com/max-sixty/worktrunk/releases/tag/v0.82.0",
                 "publishedAt": "2026-11-14T00:00:00Z", "createdAt": "2026-11-14T00:00:00Z",
                 "isPrerelease": false, "isDraft": false, "description": "## Release Notes\n"}
            ]}}}}),
        )
    }
}

/// timber's status of sluice: the stable channel at 0.22.0, though 0.22.1
/// is published (S17).
struct TimberHost;

impl StatusSource for TimberHost {
    fn status(&self, owner: &str, name: &str) -> Result<Value, String> {
        assert_eq!((owner, name), ("boomdrivedev", "sluice"));
        Ok(
            json!({"answer": "ok", "value": {"channels": {"answer": "ok", "value": {"channels": [
                {"channel": {"component": null, "name": "stable"},
                 "at": {"answer": "ok", "value": {"state": "at", "read": "source",
                        "head": {"to": {"version": "0.22.0", "index": "sha256:1"}, "at": 1795000000}, "chain": []}},
                 "notes": [{"version": "0.22.0", "index": "sha256:1", "notes": {"answer": "ok", "value": "## v0.22.0\n"}}]}
            ]}}}}),
        )
    }
}

fn options() -> Options {
    Options {
        notes_bytes: None,
        asked_at: "2026-11-19T12:00:00Z".into(),
        headwater: "0.0.0".into(),
    }
}

/// S17, C.3, C.7: a flake's tool bound to timber's stable channel reports
/// the channel's head alone, at level minor; every other GitHub input is
/// GitHub's.
#[test]
fn s17_the_binding_gives_each_identity_its_lookup() {
    let dir = tempfile::tempdir().unwrap();
    checkout(dir.path(), LOCK, Some(CONFIG));
    let report = scan(
        dir.path(),
        Lookups {
            github: GitHub,
            timber: TimberHost,
        },
        &options(),
    );

    let family = report.tiers.iter().find(|t| t.name == "family").unwrap();
    let sluice = &family.dependencies[0];
    assert_eq!(sluice.status, Status::Read);
    let newer = sluice.newer.as_ref().unwrap();
    assert_eq!(newer.len(), 1);
    assert_eq!(
        (newer[0].version.as_str(), newer[0].level.as_str()),
        ("0.22.0", "minor")
    );
    assert_eq!(newer[0].attested[0].lookup, "timber");

    let default = report.tiers.iter().find(|t| t.name == "default").unwrap();
    let worktrunk = &default.dependencies[0];
    assert_eq!(
        worktrunk.newer.as_ref().unwrap()[0].release,
        "pkg:github/max-sixty/worktrunk@0.82.0"
    );
    assert_eq!(
        report.binding.release_lookups,
        vec!["github-releases", "timber"]
    );
}

/// 51: a binding that names only GitHub's lookup leaves timber unasked.
#[test]
fn fifty_one_the_binding_names_its_lookups() {
    struct Nobody;
    impl StatusSource for Nobody {
        fn status(&self, _: &str, _: &str) -> Result<Value, String> {
            panic!("timber was asked though the binding names only GitHub's lookup")
        }
    }
    struct Both;
    impl GraphQl for Both {
        fn query(&self, _: &str) -> Result<Value, String> {
            let empty = json!({"releases": {"pageInfo": {"hasNextPage": false, "endCursor": null}, "nodes": []}});
            Ok(json!({"data": {"r0": empty.clone(), "r1": empty}}))
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let config = format!("{CONFIG}\n[binding]\nrelease_lookups = [\"github-releases\"]\n");
    checkout(dir.path(), LOCK, Some(&config));
    let report = scan(
        dir.path(),
        Lookups {
            github: Both,
            timber: Nobody,
        },
        &options(),
    );
    assert_eq!(report.binding.release_lookups, vec!["github-releases"]);
}

/// The binary says its version, revision and build, as the family's do.
#[test]
fn the_binary_names_its_version_and_revision() {
    let out = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .arg("--version")
        .output()
        .unwrap();
    let said = String::from_utf8(out.stdout).unwrap();
    assert!(said.starts_with("headwater 0.0.0, revision "), "{said}");
    assert!(said.contains(", built by rustc "), "{said}");
}

/// Section 9, millpond C.1: the binary's capture of a repository whose one
/// input no lookup reads ends in a completion naming it, and passes the
/// pack, reaching no network.
#[test]
fn c1_the_binarys_capture_passes_the_pack() {
    let dir = tempfile::tempdir().unwrap();
    let raw = tempfile::tempdir().unwrap();
    checkout(
        dir.path(),
        r#"{"nodes": {"rex": {"original": {"type": "tarball", "url": "https://example.invalid/rex.tar.gz"}},
           "root": {"inputs": {"rex": "rex"}}}, "root": "root", "version": 7}"#,
        None,
    );
    let config = dir.path().join("capture.json");
    std::fs::write(&config, json!({"repository": dir.path()}).to_string()).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(["capture", "--raw-dir"])
        .arg(raw.path())
        .env("CAPTURE_CONFIG", &config)
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        weir_pack::check_lines(&lines, weir_pack::RawPlace::Folder(raw.path()), true).is_empty()
    );
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("type tarball"));
}
