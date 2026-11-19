//! The adapter's lines, held to the capture contract by the family's
//! conformance pack (weir-pack's adapter checks; millpond C.1), and each
//! capture as millpond C.12 and headwater section 9 give it.

use std::collections::BTreeMap;
use std::path::PathBuf;

use headwater_boomdrive::run;
use headwater_core::config::Config;
use headwater_core::declaration::{Declaration, Dependency, Deps, Pin, Unread};
use headwater_core::report::Answered;
use headwater_core::seams::{Listed, Listing, NotesAnswer};
use headwater_core::{derive, Identity, Options, Reading, RepositoryState, TagScheme};
use weir_capture::{read_declarations, read_line, Line};
use weir_pack::{check_lines, RawPlace};

fn reading() -> Reading {
    let declaration = Declaration {
        description: "nix-flake".into(),
        location: "flake.lock#/nodes/sluice".into(),
        says: "github:boomdrivedev/sluice/v0.97.2".into(),
        pin: Pin::Known("0.97.2".into()),
        label: Some("sluice".into()),
        tag_scheme: Some(TagScheme::default()),
        range: None,
        claims: vec!["flake.nix:42".into()],
        upstream: Some("https://github.com/boomdrivedev/sluice".into()),
    };
    Reading {
        state: RepositoryState {
            path: PathBuf::from("/work/boomdrive"),
            commit: Some("f954b77d".into()),
            changes: false,
        },
        deps: Deps {
            dependencies: vec![
                Dependency {
                    identity: Identity::github("boomdrivedev", "sluice"),
                    declarations: vec![declaration.clone()],
                },
                Dependency {
                    identity: Identity::github("boomdrivedev", "flume"),
                    declarations: vec![Declaration {
                        location: "flake.lock#/nodes/flume".into(),
                        label: Some("flume".into()),
                        ..declaration
                    }],
                },
            ],
            unread: vec![Unread {
                description: "nix-flake".into(),
                location: "flake.lock#/nodes/rex".into(),
                reason: "the input rex is of type tarball".into(),
            }],
        },
        config: Config::default(),
        descriptions: vec!["nix-flake".into()],
    }
}

fn release(version: &str, notes: NotesAnswer) -> Listed {
    Listed {
        tag: format!("v{version}"),
        version: version.into(),
        date: Some("2026-11-19T10:12:00Z".into()),
        link: format!("https://github.com/boomdrivedev/sluice/releases/tag/v{version}"),
        prerelease: Some(false),
        withdrawn: false,
        notes: Some(("github-release".into(), notes)),
        upstream_facts: BTreeMap::new(),
    }
}

fn report() -> headwater_core::Report {
    let mut answers = BTreeMap::new();
    answers.insert(
        Identity::github("boomdrivedev", "sluice"),
        vec![Answered {
            lookup: "github-releases".into(),
            notes_lookup: Some("github-release".into()),
            listing: Listing::Listed(vec![
                release("0.97.2", NotesAnswer::Text("the pin's own".into())),
                release(
                    "0.97.5",
                    NotesAnswer::Text("## v0.97.5\r\n\r\n- a fix\r\n".into()),
                ),
                release("0.98.0", NotesAnswer::WroteNothing),
            ]),
        }],
    );
    answers.insert(
        Identity::github("boomdrivedev", "flume"),
        vec![Answered {
            lookup: "github-releases".into(),
            notes_lookup: Some("github-release".into()),
            listing: Listing::Unreached("HTTP 502".into()),
        }],
    );
    derive(
        &reading(),
        &answers,
        &Options {
            notes_bytes: None,
            asked_at: "2026-11-19T12:00:00Z".into(),
            headwater: "0.0.0".into(),
        },
    )
}

/// Section 9, millpond C.1 and C.12: one capture per newer release, its
/// raw files in the raw place, and lines the pack's adapter checks pass.
#[test]
fn c12_one_capture_per_newer_release_passes_the_pack() {
    let raw = tempfile::tempdir().unwrap();
    let mut out = Vec::new();
    run(&report()).print(raw.path(), &mut out).unwrap();
    let text = String::from_utf8(out).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    let breaches = check_lines(&lines, RawPlace::Folder(raw.path()), true);
    assert!(breaches.is_empty(), "the pack found breaches: {breaches:?}");

    let offers: Vec<_> = lines
        .iter()
        .filter_map(|l| match read_line(l).unwrap() {
            Line::Offer(offer) => Some(offer),
            _ => None,
        })
        .collect();
    let keys: Vec<&str> = offers.iter().map(|o| o.key.as_str()).collect();
    assert_eq!(
        keys,
        vec![
            "release/github/boomdrivedev/sluice/0.98.0",
            "release/github/boomdrivedev/sluice/0.97.5"
        ]
    );

    let fix = &offers[1];
    assert_eq!(fix.source, "release");
    assert_eq!(fix.event_at, "2026-11-19T10:12:00Z");
    let excerpt = fix.excerpt.as_ref().unwrap();
    assert_eq!(excerpt.text, "## v0.97.5\r\n\r\n- a fix\r\n");
    assert_eq!(excerpt.kind.as_str(), "ask");
    assert_eq!(excerpt.asserted_by, "boomdrivedev/sluice");
    assert_eq!(
        excerpt.assertion,
        "sluice 0.97.5 is released; the repository pins 0.97.2."
    );
    assert_eq!(
        excerpt.argues_with,
        vec!["flake.lock#/nodes/sluice", "flake.nix:42"]
    );
    assert_eq!(fix.facts.to_json()["level"], "patch");
    assert_eq!(
        fix.facts.to_json()["identity"],
        "pkg:github/boomdrivedev/sluice"
    );
    let notes = std::fs::read(
        raw.path()
            .join("release/github/boomdrivedev/sluice/0.97.5/notes.md"),
    )
    .unwrap();
    assert_eq!(notes, b"## v0.97.5\r\n\r\n- a fix\r\n");

    // A release with no notes is offered with no excerpt: upstream wrote
    // nothing to assert.
    assert!(offers[0].excerpt.is_none());
    assert_eq!(offers[0].facts.to_json()["level"], "minor");

    let Line::Done(done) = read_line(lines.last().unwrap()).unwrap() else {
        panic!("the last line is no completion");
    };
    let failed: Vec<&str> = done.failed.iter().map(|f| f.what.as_str()).collect();
    assert_eq!(
        failed,
        vec!["flake.lock#/nodes/rex", "pkg:github/boomdrivedev/flume"]
    );
    assert_eq!(done.listed, 4);
}

/// S6: the same report gives the same keys on every run.
#[test]
fn s6_the_same_release_keeps_its_key() {
    let one: Vec<String> = run(&report())
        .captures
        .iter()
        .map(|c| c.offer.key.as_str().to_string())
        .collect();
    let two: Vec<String> = run(&report())
        .captures
        .iter()
        .map(|c| c.offer.key.as_str().to_string())
        .collect();
    assert_eq!(one, two);
}

/// The adapter's declaration reads as the contract's.
#[test]
fn c1_the_declaration_reads() {
    let text =
        std::fs::read_to_string(format!("{}/adapter.toml", env!("CARGO_MANIFEST_DIR"))).unwrap();
    let declared = read_declarations(&text).unwrap();
    assert_eq!(declared[0].name, "headwater");
    assert_eq!(declared[0].command, vec!["headwater", "capture"]);
    assert!(declared[0].secrets.is_empty());
}
