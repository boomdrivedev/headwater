//! The version order and the level, by the dependency's own version rules
//! (6, 7, 44, 45, 70, 73; C.3), and the tag scheme that spells a version
//! as a tag (10; C.1).
//!
//! The order is Semantic Versioning 2.0.0's precedence where both versions
//! parse, as C.3 gives it for GitHub tags; any other pair is undecided
//! unless the two are spelled alike. Never a date, a name's spelling or a
//! lookup's listing order (6).

use std::cmp::Ordering;

use serde::Serialize;

/// How a dependency's tags spell its versions: the prefix that comes off a
/// tag to leave the version (C.1). `v` and the empty prefix each read a tag
/// with or without a leading `v`, since Semantic Versioning's normal form
/// has none.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct TagScheme {
    pub prefix: String,
}

impl Default for TagScheme {
    fn default() -> TagScheme {
        TagScheme {
            prefix: "v".to_string(),
        }
    }
}

impl TagScheme {
    /// The scheme and the version a ref spells as a tag, as a flake's
    /// `github:owner/repo/v1.2.3` does: what comes before its first digit
    /// is the prefix, and the rest the version. None where the ref spells
    /// no version, as a branch's name does (C.2).
    pub fn of_ref(reference: &str) -> Option<(TagScheme, String)> {
        let at = reference.find(|c: char| c.is_ascii_digit())?;
        let (prefix, rest) = reference.split_at(at);
        if prefix.contains('/') || !spells_a_version(rest) {
            return None;
        }
        let scheme = TagScheme {
            prefix: prefix.to_string(),
        };
        Some((scheme, normal(rest)))
    }

    /// The version `tag` spells under this scheme, in its normal spelling,
    /// or none where the tag is not one of the scheme's (43).
    pub fn version_of(&self, tag: &str) -> Option<String> {
        let rest = match self.prefix.as_str() {
            "" | "v" => tag.strip_prefix('v').unwrap_or(tag),
            prefix => tag.strip_prefix(prefix)?,
        };
        spells_a_version(rest).then(|| normal(rest))
    }

    /// The tag this scheme spells `version` as: the release's link (10).
    pub fn tag_of(&self, version: &str) -> String {
        format!("{}{version}", self.prefix)
    }
}

/// A version begins with a digit and holds only what a version's grammar
/// does: digits, letters, `.`, `-` and `+`.
fn spells_a_version(text: &str) -> bool {
    text.starts_with(|c: char| c.is_ascii_digit())
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+'))
}

/// Semantic Versioning's own spelling where the version parses, and the
/// text as written otherwise (C.1).
fn normal(text: &str) -> String {
    semver::Version::parse(text).map_or_else(|_| text.to_string(), |v| v.to_string())
}

/// Of two versions, which is newer (44).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    /// The first is newer.
    Newer,
    /// The second is newer.
    Older,
    Same,
    Undecided,
}

/// The order of `a` against `b`: `Newer` where `a` is later (6, 45).
pub fn order(a: &str, b: &str) -> Order {
    match (semver::Version::parse(a), semver::Version::parse(b)) {
        (Ok(a), Ok(b)) => match a.cmp_precedence(&b) {
            Ordering::Greater => Order::Newer,
            Ordering::Less => Order::Older,
            Ordering::Equal => Order::Same,
        },
        _ if a == b => Order::Same,
        _ => Order::Undecided,
    }
}

/// Whether the version's own grammar marks it a prerelease: Semantic
/// Versioning's pre-release identifiers (C.3).
pub fn grammar_marks_prerelease(version: &str) -> bool {
    semver::Version::parse(version).is_ok_and(|v| !v.pre.is_empty())
}

/// Which part of the version a release changes against the pin (70).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Major,
    Minor,
    Patch,
    Prerelease,
    Undecided,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Major => "major",
            Level::Minor => "minor",
            Level::Patch => "patch",
            Level::Prerelease => "prerelease",
            Level::Undecided => "undecided",
        }
    }
}

/// The level of `version` against `pin` (70, 73): undecided where their
/// order is, prerelease for a prerelease, and otherwise the first of the
/// major, minor and patch numbers that differs, or patch where none does.
/// Read from the two versions alone, never from notes, a date or an
/// advisory.
pub fn level(pin: &str, version: &str, prerelease: bool) -> Level {
    if order(version, pin) == Order::Undecided {
        return Level::Undecided;
    }
    if prerelease {
        return Level::Prerelease;
    }
    match (semver::Version::parse(pin), semver::Version::parse(version)) {
        (Ok(p), Ok(v)) if p.major != v.major => Level::Major,
        (Ok(p), Ok(v)) if p.minor != v.minor => Level::Minor,
        (Ok(_), Ok(_)) => Level::Patch,
        _ => Level::Undecided,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ref_spells_its_scheme_and_version() {
        let (scheme, version) = TagScheme::of_ref("v0.97.2").unwrap();
        assert_eq!(scheme.prefix, "v");
        assert_eq!(version, "0.97.2");
        let (scheme, version) = TagScheme::of_ref("cli-v1.2.3").unwrap();
        assert_eq!(scheme.prefix, "cli-v");
        assert_eq!(version, "1.2.3");
        assert_eq!(scheme.tag_of("1.2.4"), "cli-v1.2.4");
    }

    #[test]
    fn a_branch_spells_no_version() {
        assert_eq!(TagScheme::of_ref("nixos-unstable"), None);
        assert_eq!(TagScheme::of_ref("main"), None);
        assert_eq!(TagScheme::of_ref("release/1.2"), None);
    }

    #[test]
    fn a_scheme_reads_only_its_own_tags() {
        let v = TagScheme::default();
        assert_eq!(v.version_of("v1.2.3").as_deref(), Some("1.2.3"));
        assert_eq!(v.version_of("1.2.3").as_deref(), Some("1.2.3"));
        assert_eq!(v.version_of("nightly"), None);
        let cli = TagScheme {
            prefix: "cli-v".into(),
        };
        assert_eq!(cli.version_of("cli-v1.2.3").as_deref(), Some("1.2.3"));
        assert_eq!(cli.version_of("v1.2.3"), None);
    }

    #[test]
    fn the_order_is_semantic_versionings_and_undecided_otherwise() {
        assert_eq!(order("1.10.0", "1.9.0"), Order::Newer);
        assert_eq!(order("1.0.0-rc.1", "1.0.0"), Order::Older);
        assert_eq!(order("1.0.0+a", "1.0.0+b"), Order::Same);
        assert_eq!(order("2026.10", "2026.9"), Order::Undecided);
        assert_eq!(order("2026.10", "2026.10"), Order::Same);
    }

    #[test]
    fn s16_levels_follow_the_versions_alone() {
        assert_eq!(level("1.4.2", "1.4.3", false), Level::Patch);
        assert_eq!(level("1.4.2", "1.5.0", false), Level::Minor);
        assert_eq!(level("1.4.2", "2.0.0-rc.1", true), Level::Prerelease);
        assert_eq!(level("1.4.2", "2.0.0", false), Level::Major);
        assert_eq!(level("1.0.0-rc.1", "1.0.0", false), Level::Patch);
        assert_eq!(level("2026.9", "2026.10", false), Level::Undecided);
    }
}
