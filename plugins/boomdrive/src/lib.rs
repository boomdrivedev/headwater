//! headwater's Boomdrive plugin: the adapter millpond runs (statement
//! section 9; millpond C.12). On each run it prints, in the capture
//! contract the family's shared crates define (weir-capture; millpond
//! C.1), one capture for each release newer than its dependency's pin,
//! and millpond lands each one once, by its key.
//!
//! The capture of a release:
//! - its key is `release/<type>/<namespace>/<name>/<version>`, from its
//!   Package URL;
//! - its source is `release`, and its event time the release's date;
//! - its raw material holds the notes whole, and what the lookups
//!   answered;
//! - its facts are the release's own (71), with upstream's apart under
//!   `upstream-` (72);
//! - it is its own excerpt: the notes, asserted by the upstream, of kind
//!   `ask`, saying that the release is out and what the repository pins,
//!   and arguing with the declarations a move changes.
//!
//! It builds every line from one report of the core's (33), and computes
//! nothing of a release itself.

use std::path::Path;

use headwater_core::report::{DependencyReport, ReleaseReport, Status};
use headwater_core::Report;
use serde_json::{Map, Value};
use weir_capture::{Completion, Excerpt, Facts, Failure, Key, Line, Nature, Offer};

/// The source every capture names (millpond C.12).
pub const SOURCE: &str = "release";

/// A capture and the raw files it names, by their paths from the raw place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Capture {
    pub offer: Offer,
    pub raw: Vec<(String, Vec<u8>)>,
}

/// What one run offers: a capture per newer release, and each dependency
/// or declaration it could not list, with why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    pub captures: Vec<Capture>,
    pub failed: Vec<Failure>,
}

impl Run {
    /// The run's lines, in order: each offer, then the completion.
    pub fn lines(&self) -> Vec<Line> {
        self.captures
            .iter()
            .map(|c| Line::Offer(c.offer.clone()))
            .chain(std::iter::once(Line::Done(Completion {
                listed: (self.captures.len() + self.failed.len()) as u64,
                failed: self.failed.clone(),
            })))
            .collect()
    }

    /// Writes every raw file under `raw_dir`, then the lines to `out`.
    pub fn print(&self, raw_dir: &Path, out: &mut dyn std::io::Write) -> std::io::Result<()> {
        for capture in &self.captures {
            for (path, bytes) in &capture.raw {
                let file = raw_dir.join(path);
                if let Some(parent) = file.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(file, bytes)?;
            }
        }
        for line in self.lines() {
            writeln!(out, "{}", line.to_json())?;
        }
        Ok(())
    }
}

/// The run a report makes: every newer release of every dependency it
/// holds, whatever its tier.
pub fn run(report: &Report) -> Run {
    let mut captures = Vec::new();
    let mut failed: Vec<Failure> = report
        .unread_declarations
        .iter()
        .map(|u| Failure {
            what: u.location.clone(),
            why: u.reason.clone(),
        })
        .collect();
    let dependencies = report
        .tiers
        .iter()
        .flat_map(|t| t.dependencies.iter())
        .chain(report.unplaced.iter());
    for dependency in dependencies {
        if let Status::Unread { reason } = &dependency.status {
            failed.push(Failure {
                what: dependency.identity.to_string(),
                why: reason.clone(),
            });
        }
        for release in dependency.newer.iter().flatten() {
            match capture(dependency, release) {
                Ok(capture) => captures.push(capture),
                Err(why) => failed.push(Failure {
                    what: release.release.clone(),
                    why,
                }),
            }
        }
    }
    Run { captures, failed }
}

/// The key of a release (millpond C.12):
/// `release/<type>/<namespace>/<name>/<version>`.
pub fn key(dependency: &DependencyReport, release: &ReleaseReport) -> Result<Key, String> {
    let identity = &dependency.identity;
    let mut segments = vec!["release", identity.kind()];
    segments.extend(identity.namespace());
    segments.push(identity.name());
    segments.push(&release.version);
    let text = segments.join("/");
    Key::parse(&text).map_err(|e| format!("the key {text} breaks the key rule: {e:?}"))
}

fn capture(dependency: &DependencyReport, release: &ReleaseReport) -> Result<Capture, String> {
    let key = key(dependency, release)?;
    let event_at = release
        .date
        .clone()
        .ok_or("no lookup dated the release, and a capture's event time is its date")?;
    let label = dependency
        .labels
        .first()
        .cloned()
        .unwrap_or_else(|| dependency.identity.name().to_string());
    let base = key.as_str().to_string();
    let notes = release
        .notes
        .iter()
        .find_map(|n| n.text.clone().filter(|_| n.state == "text"));
    let mut raw = Vec::new();
    if let Some(text) = &notes {
        raw.push((format!("{base}/notes.md"), text.clone().into_bytes()));
    }
    let answered = serde_json::to_vec_pretty(release).expect("a release is JSON");
    raw.push((format!("{base}/release.json"), answered));
    let upstream = match dependency.identity.namespace() {
        Some(namespace) => format!("{namespace}/{}", dependency.identity.name()),
        None => dependency.identity.name().to_string(),
    };
    let pin = dependency.pin.version.clone().unwrap_or_default();
    let mut argues_with: Vec<String> = Vec::new();
    for declaration in &dependency.declarations {
        for place in std::iter::once(&declaration.location).chain(declaration.claims.iter()) {
            if !argues_with.contains(place) {
                argues_with.push(place.clone());
            }
        }
    }
    let excerpt = notes.map(|text| Excerpt {
        text,
        kind: Nature::Ask,
        asserted_by: upstream,
        assertion: format!(
            "{label} {} is released; the repository pins {pin}.",
            release.version
        ),
        subjects: vec!["release".to_string(), label.clone()],
        argues_with,
        position: "release notes".to_string(),
    });
    Ok(Capture {
        offer: Offer {
            key,
            source: SOURCE.to_string(),
            event_at,
            title: Some(format!("{label} {}", release.version)),
            url: Some(release.link.clone()),
            raw: raw.iter().map(|(path, _)| path.clone()).collect(),
            excerpt,
            facts: facts(release)?,
        },
        raw,
    })
}

/// The release's facts (71), and upstream's apart, each name under
/// `upstream-` (72), as C.9 of weir's statement spells them.
fn facts(release: &ReleaseReport) -> Result<Facts, String> {
    let mut all: Map<String, Value> = release.facts.clone();
    for stated in release.upstream_facts.values() {
        for (name, value) in stated {
            all.insert(format!("upstream-{name}"), value.clone());
        }
    }
    Facts::stated(&Value::Object(all)).map_err(|e| format!("its facts are refused: {e}"))
}
