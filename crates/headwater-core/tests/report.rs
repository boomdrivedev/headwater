//! The derivation against fixed readings and fixed lookup answers. Each
//! case names the clauses of the statement it holds the core to.

use std::collections::BTreeMap;
use std::path::PathBuf;

use headwater_core::config::Config;
use headwater_core::declaration::{Declaration, Dependency, Deps, Pin, Unread};
use headwater_core::report::{Answered, Status};
use headwater_core::seams::{Listed, Listing, NotesAnswer};
use headwater_core::{
    derive, Identity, Level, Options, Reading, Report, RepositoryState, TagScheme,
};

fn declared(location: &str, pin: Pin, label: &str, range: Option<&str>) -> Declaration {
    Declaration {
        description: "fixture".into(),
        location: location.into(),
        says: String::new(),
        pin,
        label: Some(label.into()),
        tag_scheme: Some(TagScheme::default()),
        range: range.map(str::to_string),
        claims: vec![format!("{location}.claim")],
        upstream: None,
    }
}

fn dependency(owner: &str, repo: &str, declarations: Vec<Declaration>) -> Dependency {
    Dependency {
        identity: Identity::github(owner, repo),
        declarations,
    }
}

fn reading(dependencies: Vec<Dependency>, config: &str) -> Reading {
    Reading {
        state: RepositoryState {
            path: PathBuf::from("/work/fixture"),
            commit: Some("4f2e9c1".into()),
            changes: false,
        },
        deps: Deps {
            dependencies,
            unread: vec![Unread {
                description: "nix-flake".into(),
                location: "flake.lock#/nodes/rex".into(),
                reason: "the input rex is of type tarball".into(),
            }],
        },
        config: Config::parse(config).unwrap(),
        descriptions: vec!["fixture".into()],
    }
}

fn release(version: &str, prerelease: bool, notes: NotesAnswer) -> Listed {
    Listed {
        tag: format!("v{version}"),
        version: version.into(),
        date: Some("2026-10-12T10:02:00Z".into()),
        link: format!("https://example.invalid/v{version}"),
        prerelease: Some(prerelease),
        withdrawn: false,
        notes: Some(("fixture-notes".into(), notes)),
        upstream_facts: BTreeMap::new(),
    }
}

fn listed(releases: Vec<Listed>) -> Vec<Answered> {
    vec![Answered {
        lookup: "fixture".into(),
        notes_lookup: Some("fixture-notes".into()),
        listing: Listing::Listed(releases),
    }]
}

fn options(notes_bytes: Option<usize>) -> Options {
    Options {
        notes_bytes,
        asked_at: "2026-10-17T08:00:00Z".into(),
        headwater: "0.0.0".into(),
    }
}

fn find<'a>(report: &'a Report, identity: &str) -> &'a headwater_core::report::DependencyReport {
    report
        .tiers
        .iter()
        .flat_map(|t| t.dependencies.iter())
        .chain(report.unplaced.iter())
        .find(|d| d.identity.to_string() == identity)
        .unwrap_or_else(|| panic!("{identity} is not in the report"))
}

fn text(notes: &str) -> NotesAnswer {
    NotesAnswer::Text(notes.into())
}

/// S16, 70, 8, 6: levels from the versions alone, and a newer prerelease
/// held back where neither the pin nor the tier asks for it.
#[test]
fn s16_levels_and_prereleases() {
    let r = reading(
        vec![dependency(
            "acme",
            "ui",
            vec![declared("a", Pin::Known("1.4.2".into()), "ui", None)],
        )],
        "",
    );
    let mut answers = BTreeMap::new();
    answers.insert(
        Identity::github("acme", "ui"),
        listed(vec![
            release("1.4.1", false, text("old")),
            release("1.4.2", false, text("the pin's own")),
            release("2.0.0-rc.1", true, text("rc")),
            release("1.5.0", false, text("BREAKING: everything changed")),
            release("1.4.3", false, text("fix")),
        ]),
    );
    let report = derive(&r, &answers, &options(None));
    let ui = find(&report, "pkg:github/acme/ui");
    let newer: Vec<(&str, Level)> = ui
        .newer
        .as_ref()
        .unwrap()
        .iter()
        .map(|r| (r.version.as_str(), r.level))
        .collect();
    assert_eq!(
        newer,
        vec![("1.5.0", Level::Minor), ("1.4.3", Level::Patch)]
    );
    assert_eq!(ui.prereleases_left_out, 1);
    assert_eq!(ui.status, Status::Read);

    // The tier that asks for prereleases holds it, at level prerelease.
    let r = reading(
        r.deps.dependencies.clone(),
        "[tiers.favorites]\nidentities = [\"pkg:github/acme/ui\"]\nprereleases = true\n",
    );
    let report = derive(&r, &answers, &options(None));
    let ui = find(&report, "pkg:github/acme/ui");
    let first = &ui.newer.as_ref().unwrap()[0];
    assert_eq!(
        (first.version.as_str(), first.level),
        ("2.0.0-rc.1", Level::Prerelease)
    );
    assert_eq!(ui.tier.as_deref(), Some("favorites"));
}

/// 3, 4, 58, 1: an unknown pin is never "nothing newer"; an identity no
/// lookup knows is said so; an unread declaration is in every report.
#[test]
fn three_unknown_pins_and_fifty_eight_unknown_identities() {
    let r = reading(
        vec![
            dependency(
                "nixos",
                "nixpkgs",
                vec![declared(
                    "n",
                    Pin::Unknown("a branch".into()),
                    "nixpkgs",
                    None,
                )],
            ),
            dependency(
                "acme",
                "lost",
                vec![declared("l", Pin::Known("1.0.0".into()), "lost", None)],
            ),
        ],
        "",
    );
    let mut answers = BTreeMap::new();
    answers.insert(
        Identity::github("nixos", "nixpkgs"),
        listed(vec![release("25.5.0", false, NotesAnswer::WroteNothing)]),
    );
    let report = derive(&r, &answers, &options(None));
    let nixpkgs = find(&report, "pkg:github/nixos/nixpkgs");
    assert_eq!(nixpkgs.newer, None);
    assert_eq!(nixpkgs.newest.as_ref().unwrap().version, "25.5.0");
    assert!(nixpkgs.pin.unknown.as_deref().unwrap().contains("a branch"));
    let lost = find(&report, "pkg:github/acme/lost");
    assert_eq!(lost.status, Status::KnownToNoLookup);
    assert_eq!(lost.newer, None);
    assert_eq!(report.unread_declarations.len(), 1);
}

/// 16, 55, S3: an unreached lookup makes the dependency unread with the
/// reason, and other dependencies are reported as usual.
#[test]
fn s3_an_unreached_lookup_is_unread_with_its_reason() {
    let r = reading(
        vec![
            dependency(
                "acme",
                "down",
                vec![declared("d", Pin::Known("1.0.0".into()), "down", None)],
            ),
            dependency(
                "acme",
                "up",
                vec![declared("u", Pin::Known("1.0.0".into()), "up", None)],
            ),
        ],
        "",
    );
    let mut answers = BTreeMap::new();
    answers.insert(
        Identity::github("acme", "down"),
        vec![Answered {
            lookup: "fixture".into(),
            notes_lookup: None,
            listing: Listing::Unreached("HTTP 502".into()),
        }],
    );
    answers.insert(
        Identity::github("acme", "up"),
        listed(vec![release(
            "1.0.1",
            false,
            NotesAnswer::Unread("timed out".into()),
        )]),
    );
    let report = derive(&r, &answers, &options(None));
    let down = find(&report, "pkg:github/acme/down");
    assert!(matches!(&down.status, Status::Unread { reason } if reason.contains("HTTP 502")));
    assert_eq!(down.newer, None);
    let up = find(&report, "pkg:github/acme/up");
    let notes = &up.newer.as_ref().unwrap()[0].notes[0];
    assert_eq!(
        (notes.state, notes.reason.as_deref()),
        ("unread", Some("timed out"))
    );
}

/// 7: a release whose order against the pin cannot be decided is beside the
/// newer ones, never among them and never left out.
#[test]
fn seven_undecided_releases_are_given_apart() {
    let r = reading(
        vec![dependency(
            "acme",
            "cal",
            vec![declared("c", Pin::Known("1.0.0".into()), "cal", None)],
        )],
        "",
    );
    let mut answers = BTreeMap::new();
    answers.insert(
        Identity::github("acme", "cal"),
        listed(vec![release("2026.10", false, text("calendar"))]),
    );
    let report = derive(&r, &answers, &options(None));
    let cal = find(&report, "pkg:github/acme/cal");
    assert_eq!(cal.newer.as_ref().unwrap().len(), 0);
    assert_eq!(cal.undecided[0].version, "2026.10");
    assert_eq!(cal.undecided[0].level, Level::Undecided);
}

/// 4, 26, 56, S13: the oldest declared pin decides, and each release is
/// marked inside or outside each declaration's range.
#[test]
fn s13_ranges_mark_releases_and_the_oldest_pin_decides() {
    let r = reading(
        vec![dependency(
            "acme",
            "client",
            vec![
                declared(
                    "pyproject",
                    Pin::Known("3.2.0".into()),
                    "client",
                    Some("^3.0"),
                ),
                declared("tools", Pin::Known("3.1.0".into()), "client", None),
            ],
        )],
        "",
    );
    let mut answers = BTreeMap::new();
    answers.insert(
        Identity::github("acme", "client"),
        listed(vec![
            release("3.2.0", false, text("minor")),
            release("4.0.0", false, text("major")),
        ]),
    );
    let report = derive(&r, &answers, &options(None));
    let client = find(&report, "pkg:github/acme/client");
    assert_eq!(client.pin.version.as_deref(), Some("3.1.0"));
    let marks: Vec<(&str, bool)> = client
        .newer
        .as_ref()
        .unwrap()
        .iter()
        .map(|r| (r.version.as_str(), r.ranges[0].inside))
        .collect();
    assert_eq!(marks, vec![("4.0.0", false), ("3.2.0", true)]);
}

/// 12, S7, 11: notes whole and unchanged, or the leading portion asked for
/// cut at a line's end, with the whole's size.
#[test]
fn s7_notes_whole_or_a_portion() {
    let long: String = (0..4096)
        .map(|n| format!("line {n} of the notes\n"))
        .collect();
    let r = reading(
        vec![dependency(
            "acme",
            "big",
            vec![declared("b", Pin::Known("1.0.0".into()), "big", None)],
        )],
        "",
    );
    let mut answers = BTreeMap::new();
    answers.insert(
        Identity::github("acme", "big"),
        listed(vec![release("1.1.0", false, text(&long))]),
    );
    let whole = derive(&r, &answers, &options(None));
    let notes = &find(&whole, "pkg:github/acme/big").newer.as_ref().unwrap()[0].notes[0];
    assert_eq!(notes.text.as_deref(), Some(long.as_str()));
    assert_eq!(notes.whole, Some(true));
    assert!(long.len() > 40 * 1024);

    let part = derive(&r, &answers, &options(Some(1000)));
    let notes = &find(&part, "pkg:github/acme/big").newer.as_ref().unwrap()[0].notes[0];
    let given = notes.text.as_deref().unwrap();
    assert!(given.len() <= 1000 && given.ends_with('\n') && long.starts_with(given));
    assert_eq!((notes.whole, notes.bytes), (Some(false), Some(long.len())));
}

/// 10, 71, 53: a release's identity is its Package URL with its version,
/// and its facts are read from the reading and the lookup alone.
#[test]
fn seventy_one_facts_and_ten_identity() {
    let r = reading(
        vec![dependency(
            "Acme",
            "UI",
            vec![declared("a", Pin::Known("1.0.0".into()), "ui", None)],
        )],
        "",
    );
    let mut answers = BTreeMap::new();
    answers.insert(
        Identity::github("acme", "ui"),
        listed(vec![release("1.0.1", false, text("x"))]),
    );
    let report = derive(&r, &answers, &options(None));
    let release = &find(&report, "pkg:github/acme/ui").newer.as_ref().unwrap()[0];
    assert_eq!(release.release, "pkg:github/acme/ui@1.0.1");
    let facts = serde_json::Value::Object(release.facts.clone());
    assert_eq!(
        facts,
        serde_json::json!({
            "identity": "pkg:github/acme/ui", "label": ["ui"], "tier": "default",
            "pin": "1.0.0", "version": "1.0.1", "level": "patch", "date": "2026-10-12",
            "advisories": "unread", "lookup": ["fixture"]
        })
    );
}

/// 57, S14: two tiers naming one identity alike leave it unread naming
/// both, and every other dependency is reported as usual.
#[test]
fn s14_two_tiers_alike() {
    let r = reading(
        vec![
            dependency("acme", "twin", vec![declared("t", Pin::Known("1.0.0".into()), "twin", None)]),
            dependency("acme", "solo", vec![declared("s", Pin::Known("1.0.0".into()), "solo", None)]),
        ],
        "[tiers.a]\nidentities = [\"pkg:github/acme/twin\"]\n[tiers.b]\nidentities = [\"pkg:github/acme/twin\"]\n",
    );
    let report = derive(&r, &BTreeMap::new(), &options(None));
    let twin = find(&report, "pkg:github/acme/twin");
    assert!(matches!(&twin.status, Status::Unread { reason } if reason.contains("a and b")));
    assert_eq!(report.unplaced.len(), 1);
    assert_eq!(
        find(&report, "pkg:github/acme/solo").tier.as_deref(),
        Some("default")
    );
}

/// 32: the same reading, answers and options give the same report.
#[test]
fn thirty_two_the_derivation_is_deterministic() {
    let r = reading(
        vec![dependency(
            "acme",
            "ui",
            vec![declared("a", Pin::Known("1.0.0".into()), "ui", None)],
        )],
        "",
    );
    let mut answers = BTreeMap::new();
    answers.insert(
        Identity::github("acme", "ui"),
        listed(vec![
            release("1.0.2", false, text("b")),
            release("1.0.1", false, text("a")),
        ]),
    );
    let one = serde_json::to_string(&derive(&r, &answers, &options(None))).unwrap();
    let two = serde_json::to_string(&derive(&r, &answers, &options(None))).unwrap();
    assert_eq!(one, two);
}
