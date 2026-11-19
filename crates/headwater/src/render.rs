//! The report for a person: a projection of the JSON form, holding
//! nothing it does not (35). Each release's notes are given in the leading
//! portion asked for, cut at a line's end, with their whole size (12).

use std::fmt::Write;

use headwater_core::report::{DependencyReport, ReleaseReport, Status};
use headwater_core::Report;

/// The report as text.
pub fn text(report: &Report) -> String {
    let mut out = String::new();
    let commit = report.repository.commit.as_deref().unwrap_or("no commit");
    let changes = if report.repository.changes {
        ", with uncommitted changes"
    } else {
        ""
    };
    let _ = writeln!(
        out,
        "{} at {}{changes}, asked {}",
        report.repository.path.display(),
        short(commit),
        report.asked_at
    );
    for unread in &report.unread_declarations {
        let _ = writeln!(out, "unread  {}: {}", unread.location, unread.reason);
    }
    for dependency in &report.unplaced {
        dependency_text(&mut out, dependency);
    }
    for tier in &report.tiers {
        let _ = writeln!(out, "\ntier {} (cadence {})", tier.name, tier.cadence);
        for dependency in &tier.dependencies {
            dependency_text(&mut out, dependency);
        }
    }
    out
}

fn short(commit: &str) -> &str {
    commit.get(..12).unwrap_or(commit)
}

fn dependency_text(out: &mut String, dependency: &DependencyReport) {
    let label = dependency.labels.join(", ");
    let pin = match (&dependency.pin.version, &dependency.pin.unknown) {
        (Some(version), _) => format!("pins {version}"),
        (None, Some(why)) => format!("pin unknown ({why})"),
        (None, None) => "pin unknown".to_string(),
    };
    let state = match (&dependency.status, &dependency.newer) {
        (Status::KnownToNoLookup, _) => "known to no lookup".to_string(),
        (Status::Unread { reason }, _) => format!("unread: {reason}"),
        (Status::Read, Some(newer)) if newer.is_empty() => "nothing newer".to_string(),
        (Status::Read, Some(newer)) => format!("{} newer", newer.len()),
        (Status::Read, None) => "no newer releases can be told".to_string(),
    };
    let _ = writeln!(out, "\n  {label}  {}  {pin}  {state}", dependency.identity);
    for release in dependency.newer.iter().flatten() {
        release_text(out, release);
    }
    for release in &dependency.undecided {
        let _ = write!(out, "    (order undecided) ");
        release_text(out, release);
    }
    if let Some(newest) = &dependency.newest {
        let _ = writeln!(out, "    newest listed: {} {}", newest.version, newest.link);
    }
    if dependency.prereleases_left_out > 0 {
        let _ = writeln!(
            out,
            "    {} newer prereleases left out: neither the pin nor the tier asks for them",
            dependency.prereleases_left_out
        );
    }
}

fn release_text(out: &mut String, release: &ReleaseReport) {
    let date = release
        .date
        .as_deref()
        .and_then(|d| d.get(..10))
        .unwrap_or("undated");
    let _ = writeln!(
        out,
        "    {:<14} {:<10} {date}  {}",
        release.version,
        release.level.as_str(),
        release.link
    );
    for notes in &release.notes {
        match (notes.state, &notes.text) {
            ("text", Some(text)) => {
                for line in text.lines() {
                    let _ = writeln!(out, "      | {line}");
                }
                if notes.whole == Some(false) {
                    let _ = writeln!(
                        out,
                        "      | ... ({} bytes in all; --notes-bytes 0 gives them whole)",
                        notes.bytes.unwrap_or_default()
                    );
                }
            }
            ("wrote_nothing", _) => {
                let _ = writeln!(out, "      | (upstream wrote no notes)");
            }
            (_, _) => {
                let _ = writeln!(
                    out,
                    "      | (notes unread: {})",
                    notes.reason.as_deref().unwrap_or("no reason given")
                );
            }
        }
    }
}
