//! The derivation: a report from one reading and what the lookups answered
//! (model section 3). It reaches no lookup and writes nothing; the same
//! reading, answers and options give the same report, byte for byte (32,
//! D12).

use std::cmp::Ordering;
use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

use crate::config::Placed;
use crate::declaration::{Declaration, Dependency, Pin, Unread};
use crate::identity::Identity;
use crate::reading::{Reading, RepositoryState};
use crate::seams::{Listed, Listing, NotesAnswer};
use crate::version::{grammar_marks_prerelease, level, order, Level, Order};

/// What the asker chose (model section 1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    /// The leading portion of each release's notes to give, in bytes, cut
    /// at the end of a line; whole where none (12).
    pub notes_bytes: Option<usize>,
    /// When the report was asked for, in RFC 3339.
    pub asked_at: String,
    /// headwater's own version (32).
    pub headwater: String,
}

/// One lookup's answer for one identity, by the lookup's name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answered {
    pub lookup: String,
    pub notes_lookup: Option<String>,
    pub listing: Listing,
}

/// The lookups asked and the descriptions read, as a report names them.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct BindingReport {
    pub dependency_descriptions: Vec<String>,
    pub release_lookups: Vec<String>,
    pub notes_lookups: Vec<String>,
    pub advisory_lookups: Vec<String>,
}

/// The answer of a scan (35): one form for every surface.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Report {
    pub answer: &'static str,
    pub headwater: String,
    pub repository: RepositoryState,
    pub asked_at: String,
    pub binding: BindingReport,
    /// Declarations that could not be read, whatever tiers were asked (1).
    pub unread_declarations: Vec<Unread>,
    /// Dependencies two tiers name alike, reported unread (57).
    pub unplaced: Vec<DependencyReport>,
    pub tiers: Vec<TierReport>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TierReport {
    pub name: String,
    pub cadence: String,
    pub prereleases: bool,
    pub dependencies: Vec<DependencyReport>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Status {
    Read,
    Unread { reason: String },
    KnownToNoLookup,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PinReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LookupReport {
    pub lookup: String,
    pub answer: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub releases: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdvisoriesReport {
    pub status: &'static str,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DependencyReport {
    pub identity: Identity,
    pub labels: Vec<String>,
    /// Its tier, or none where two tiers name it alike (57).
    pub tier: Option<String>,
    #[serde(flatten)]
    pub status: Status,
    pub declarations: Vec<Declaration>,
    pub pin: PinReport,
    pub lookups: Vec<LookupReport>,
    pub advisories: AdvisoriesReport,
    /// Every release newer than the pin: null where the pin is unknown or
    /// no lookup that knows it answered, never an empty list then (3).
    pub newer: Option<Vec<ReleaseReport>>,
    /// Releases whose order against the pin is undecided (7).
    pub undecided: Vec<ReleaseReport>,
    /// The newest release listed, given where the pin is unknown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newest: Option<ReleaseReport>,
    /// Newer prereleases a report does not hold, since neither the pin
    /// nor the tier asks for them (8).
    pub prereleases_left_out: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RangeMark {
    pub declaration: String,
    pub range: String,
    pub inside: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NotesReport {
    pub lookup: String,
    pub state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub whole: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// One lookup's account of a release (39).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Attested {
    pub lookup: String,
    pub tag: String,
    pub date: Option<String>,
    pub link: String,
    pub withdrawn: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ReleaseReport {
    /// The release's identity, for ever (10).
    pub release: String,
    pub version: String,
    pub level: Level,
    pub prerelease: bool,
    pub date: Option<String>,
    pub link: String,
    pub withdrawn: bool,
    pub attested: Vec<Attested>,
    pub ranges: Vec<RangeMark>,
    pub notes: Vec<NotesReport>,
    /// headwater's own facts of the release, for a program to match (71).
    pub facts: Map<String, Value>,
    /// What upstream states of it, apart, by lookup (72).
    pub upstream_facts: BTreeMap<String, BTreeMap<String, Value>>,
}

/// The report of `reading` from `answers`, each identity's lookups'
/// answers; an identity with none is known to no lookup (58).
pub fn derive(
    reading: &Reading,
    answers: &BTreeMap<Identity, Vec<Answered>>,
    options: &Options,
) -> Report {
    let mut tiers: BTreeMap<String, TierReport> = BTreeMap::new();
    let mut unplaced = Vec::new();
    for dependency in &reading.deps.dependencies {
        let none = Vec::new();
        let answered = answers.get(&dependency.identity).unwrap_or(&none);
        match reading.config.place(&dependency.identity) {
            Placed::Tier { name, prereleases } => {
                let report =
                    dependency_report(dependency, Some(&name), prereleases, answered, options);
                tiers
                    .entry(name.clone())
                    .or_insert_with(|| TierReport {
                        cadence: reading.config.cadence(&name).to_string(),
                        name,
                        prereleases,
                        dependencies: vec![],
                    })
                    .dependencies
                    .push(report);
            }
            Placed::NamedAlike(names) => {
                let mut report = dependency_report(dependency, None, false, answered, options);
                report.status = Status::Unread {
                    reason: format!("named alike by the tiers {} (57)", names.join(" and ")),
                };
                unplaced.push(report);
            }
        }
    }
    let lookups: Vec<&Answered> = answers.values().flatten().collect();
    let mut release_lookups: Vec<String> = lookups.iter().map(|a| a.lookup.clone()).collect();
    release_lookups.sort();
    release_lookups.dedup();
    let mut notes_lookups: Vec<String> = lookups
        .iter()
        .filter_map(|a| a.notes_lookup.clone())
        .collect();
    notes_lookups.sort();
    notes_lookups.dedup();
    Report {
        answer: "scan",
        headwater: options.headwater.clone(),
        repository: reading.state.clone(),
        asked_at: options.asked_at.clone(),
        binding: BindingReport {
            dependency_descriptions: reading.descriptions.clone(),
            release_lookups,
            notes_lookups,
            advisory_lookups: vec![],
        },
        unread_declarations: reading.deps.unread.clone(),
        unplaced,
        tiers: tiers.into_values().collect(),
    }
}

/// A release as every lookup that lists it gave it, merged by version (39).
struct Merged<'a> {
    version: String,
    accounts: Vec<(&'a Answered, &'a Listed)>,
}

fn dependency_report(
    dependency: &Dependency,
    tier: Option<&str>,
    tier_prereleases: bool,
    answered: &[Answered],
    options: &Options,
) -> DependencyReport {
    let pin = dependency.pin();
    let labels = dependency.labels();
    let lookups: Vec<LookupReport> = answered
        .iter()
        .map(|a| match &a.listing {
            Listing::Listed(releases) => LookupReport {
                lookup: a.lookup.clone(),
                answer: "listed",
                releases: Some(releases.len()),
                reason: None,
            },
            Listing::Unreached(reason) => LookupReport {
                lookup: a.lookup.clone(),
                answer: "unreached",
                releases: None,
                reason: Some(reason.clone()),
            },
        })
        .collect();
    let unreached: Vec<String> = answered
        .iter()
        .filter_map(|a| match &a.listing {
            Listing::Unreached(reason) => Some(format!("{}: {reason}", a.lookup)),
            Listing::Listed(_) => None,
        })
        .collect();
    let status = if answered.is_empty() {
        Status::KnownToNoLookup
    } else if !unreached.is_empty() {
        Status::Unread {
            reason: unreached.join("; "),
        }
    } else {
        Status::Read
    };
    let any_listed = answered
        .iter()
        .any(|a| matches!(a.listing, Listing::Listed(_)));

    let mut merged: Vec<Merged> = Vec::new();
    for a in answered {
        if let Listing::Listed(releases) = &a.listing {
            for listed in releases {
                match merged.iter_mut().find(|m| m.version == listed.version) {
                    Some(m) => m.accounts.push((a, listed)),
                    None => merged.push(Merged {
                        version: listed.version.clone(),
                        accounts: vec![(a, listed)],
                    }),
                }
            }
        }
    }

    let context = Context {
        dependency,
        labels: &labels,
        tier,
        pin: pin.version(),
        options,
    };
    let mut newer = Vec::new();
    let mut undecided = Vec::new();
    let mut prereleases_left_out = 0;
    if let Pin::Known(pin_version) = &pin {
        let pin_is_prerelease = grammar_marks_prerelease(pin_version);
        for m in &merged {
            let release = context.release(m);
            match order(&m.version, pin_version) {
                Order::Newer if release.prerelease && !(pin_is_prerelease || tier_prereleases) => {
                    prereleases_left_out += 1
                }
                Order::Newer => newer.push(release),
                Order::Undecided => undecided.push(release),
                Order::Same | Order::Older => {}
            }
        }
    }
    newer.sort_by(|a, b| newest_first(&a.version, &b.version));
    undecided.sort_by(|a, b| a.version.cmp(&b.version));
    let newest = match &pin {
        Pin::Unknown(_) => merged
            .iter()
            .filter(|m| semver::Version::parse(&m.version).is_ok())
            .max_by(|a, b| newest_first(&b.version, &a.version))
            .map(|m| context.release(m)),
        Pin::Known(_) => None,
    };
    DependencyReport {
        identity: dependency.identity.clone(),
        labels,
        tier: tier.map(str::to_string),
        status,
        declarations: dependency.declarations.clone(),
        pin: match &pin {
            Pin::Known(version) => PinReport {
                version: Some(version.clone()),
                unknown: None,
            },
            Pin::Unknown(why) => PinReport {
                version: None,
                unknown: Some(why.clone()),
            },
        },
        lookups,
        advisories: AdvisoriesReport {
            status: "unread",
            reason: "no advisory lookup is bound".to_string(),
        },
        newer: (matches!(pin, Pin::Known(_)) && any_listed).then_some(newer),
        undecided,
        newest,
        prereleases_left_out,
    }
}

/// Newest first by Semantic Versioning's precedence, then by spelling.
fn newest_first(a: &str, b: &str) -> Ordering {
    match order(a, b) {
        Order::Newer => Ordering::Less,
        Order::Older => Ordering::Greater,
        Order::Same | Order::Undecided => a.cmp(b),
    }
}

struct Context<'a> {
    dependency: &'a Dependency,
    labels: &'a [String],
    tier: Option<&'a str>,
    pin: Option<&'a str>,
    options: &'a Options,
}

impl Context<'_> {
    fn release(&self, merged: &Merged) -> ReleaseReport {
        let version = merged.version.clone();
        let marks: Vec<bool> = merged
            .accounts
            .iter()
            .filter_map(|(_, l)| l.prerelease)
            .collect();
        // The lookup's own mark, and the version's grammar where no lookup
        // gives one (vocabulary: *prerelease*).
        let prerelease = if marks.is_empty() {
            grammar_marks_prerelease(&version)
        } else {
            marks.iter().any(|m| *m)
        };
        let level = self
            .pin
            .map_or(Level::Undecided, |pin| level(pin, &version, prerelease));
        let (_, first) = merged.accounts[0];
        let date = merged.accounts.iter().find_map(|(_, l)| l.date.clone());
        let withdrawn = merged.accounts.iter().any(|(_, l)| l.withdrawn);
        let attested: Vec<Attested> = merged
            .accounts
            .iter()
            .map(|(a, l)| Attested {
                lookup: a.lookup.clone(),
                tag: l.tag.clone(),
                date: l.date.clone(),
                link: l.link.clone(),
                withdrawn: l.withdrawn,
            })
            .collect();
        let ranges: Vec<RangeMark> = self
            .dependency
            .declarations
            .iter()
            .filter_map(|d| {
                let range = d.range.as_ref()?;
                let req = semver::VersionReq::parse(range).ok()?;
                let inside = semver::Version::parse(&version).is_ok_and(|v| req.matches(&v));
                Some(RangeMark {
                    declaration: d.location.clone(),
                    range: range.clone(),
                    inside,
                })
            })
            .collect();
        let notes: Vec<NotesReport> = merged
            .accounts
            .iter()
            .filter_map(|(_, l)| l.notes.as_ref())
            .map(|(lookup, answer)| notes_report(lookup, answer, self.options.notes_bytes))
            .collect();
        let upstream_facts: BTreeMap<String, BTreeMap<String, Value>> = merged
            .accounts
            .iter()
            .filter(|(_, l)| !l.upstream_facts.is_empty())
            .map(|(a, l)| (a.lookup.clone(), l.upstream_facts.clone()))
            .collect();
        let lookups: Vec<String> = merged
            .accounts
            .iter()
            .map(|(a, _)| a.lookup.clone())
            .collect();
        let mut facts = Map::new();
        facts.insert(
            "identity".into(),
            json!(self.dependency.identity.to_string()),
        );
        if !self.labels.is_empty() {
            facts.insert("label".into(), json!(self.labels));
        }
        if let Some(tier) = self.tier {
            facts.insert("tier".into(), json!(tier));
        }
        if let Some(pin) = self.pin {
            facts.insert("pin".into(), json!(pin));
        }
        facts.insert("version".into(), json!(version));
        facts.insert("level".into(), json!(level.as_str()));
        if let Some(day) = date.as_deref().and_then(|d| d.get(..10)) {
            facts.insert("date".into(), json!(day));
        }
        facts.insert("advisories".into(), json!("unread"));
        facts.insert("lookup".into(), json!(lookups));
        ReleaseReport {
            release: self.dependency.identity.release(&version),
            version,
            level,
            prerelease,
            date,
            link: first.link.clone(),
            withdrawn,
            attested,
            ranges,
            notes,
            facts,
            upstream_facts,
        }
    }
}

/// The notes as a report gives them: whole, or the leading portion asked
/// for, cut at the end of a line, with the whole's size and digest (12).
pub fn notes_report(lookup: &str, answer: &NotesAnswer, part: Option<usize>) -> NotesReport {
    let base = NotesReport {
        lookup: lookup.to_string(),
        state: "",
        text: None,
        bytes: None,
        sha256: None,
        whole: None,
        reason: None,
    };
    match answer {
        NotesAnswer::Text(text) => {
            let given = part.map_or(text.as_str(), |n| cut(text, n));
            NotesReport {
                state: "text",
                text: Some(given.to_string()),
                bytes: Some(text.len()),
                sha256: Some(hex(&Sha256::digest(text.as_bytes()))),
                whole: Some(given.len() == text.len()),
                ..base
            }
        }
        NotesAnswer::WroteNothing => NotesReport {
            state: "wrote_nothing",
            ..base
        },
        NotesAnswer::Unread(reason) => NotesReport {
            state: "unread",
            reason: Some(reason.clone()),
            ..base
        },
    }
}

/// The leading portion of `text` within `n` bytes, cut back to the end of
/// its last whole line; the whole where it fits (12).
pub fn cut(text: &str, n: usize) -> &str {
    if text.len() <= n {
        return text;
    }
    let mut end = n;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    match text[..end].rfind('\n') {
        Some(at) => &text[..=at],
        None => "",
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twelve_a_portion_is_cut_at_a_line_end() {
        let text = "one\ntwo\nthree\n";
        assert_eq!(cut(text, 100), text);
        assert_eq!(cut(text, 9), "one\ntwo\n");
        assert_eq!(cut(text, 2), "");
        assert_eq!(cut("é\nb", 1), "");
    }
}
