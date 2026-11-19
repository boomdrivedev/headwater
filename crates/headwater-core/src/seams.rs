//! The release lookup's seam (B.3, B.4), as a trait each profile
//! implements (model section 6). A lookup is asked for many identities at
//! once, and learns nothing of the repository but them and their tag
//! schemes (37). A profile that reads notes in the same answer, as GitHub's
//! release bodies come with their releases, gives each release's notes
//! under its notes lookup's name (C.4).

use std::collections::BTreeMap;

use serde::Serialize;

use crate::identity::Identity;
use crate::version::TagScheme;

/// What a lookup is asked of one identity: the identity, its tag scheme
/// where a declaration states one (43), and where upstream keeps its
/// releases where a declaration states it (68).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
    pub identity: Identity,
    pub tag_scheme: TagScheme,
}

/// What upstream wrote of a release, whole and unchanged; that it wrote
/// nothing; or that the notes lookup could not be read (46, 55).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NotesAnswer {
    Text(String),
    WroteNothing,
    Unread(String),
}

/// One release as a lookup lists it (43).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Listed {
    /// The tag the release is at: its link's name.
    pub tag: String,
    /// The version in its normal spelling (10; C.1).
    pub version: String,
    /// When it was published, in RFC 3339, where the lookup says.
    pub date: Option<String>,
    pub link: String,
    /// The lookup's own prerelease mark, where it gives one (43).
    pub prerelease: Option<bool>,
    pub withdrawn: bool,
    /// The notes, read in the same answer by the notes lookup named.
    pub notes: Option<(String, NotesAnswer)>,
    /// The facts upstream states of the release, where the lookup gives
    /// any (72).
    pub upstream_facts: BTreeMap<String, serde_json::Value>,
}

/// A lookup's answer for one identity (43).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Listing {
    Listed(Vec<Listed>),
    Unreached(String),
}

/// A release lookup (43, 44, 68, 73). The order and the level are the
/// core's, by Semantic Versioning, for every profile of this version
/// (`crate::version`).
pub trait ReleaseLookup {
    /// The profile's name, as a binding and a report give it.
    fn name(&self) -> &str;
    /// Every release it lists for each identity asked, in the order asked.
    fn list_releases(&self, queries: &[Query]) -> Vec<(Identity, Listing)>;
}
