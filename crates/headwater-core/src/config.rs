//! The tiers declaration, `.config/headwater.toml` (C.7): each tier's name,
//! its cadence, the identities it names outright or by a prefix and whether
//! it asks for prereleases, with the default tier's cadence (41); and the
//! binding: the lookups named, and the identities the timber lookup
//! answers for, each with the channel it reads (67).
//!
//! ```toml
//! [default]
//! cadence = "7d"
//!
//! [tiers.family]
//! cadence = "1d"
//! prefixes = ["pkg:github/boomdrivedev/"]
//! prereleases = false
//!
//! [binding]
//! release_lookups = ["github-releases", "timber"]
//!
//! [[timber]]
//! identities = ["pkg:github/boomdrivedev/sluice"]
//! channel = "stable"
//! ```
//!
//! A repository with no such file has the default tier alone, with a
//! cadence of seven days, and binds every profile that recognises one of
//! its declarations (51).

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::identity::Identity;

/// Where the file stands in a repository state.
pub const PATH: &str = ".config/headwater.toml";

/// The default tier's name and cadence where the file sets none (C.7).
pub const DEFAULT_TIER: &str = "default";
pub const DEFAULT_CADENCE: &str = "7d";

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub default: DefaultTier,
    #[serde(default)]
    pub tiers: BTreeMap<String, TierDeclaration>,
    #[serde(default)]
    pub binding: Binding,
    #[serde(default)]
    pub timber: Vec<TimberBinding>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefaultTier {
    #[serde(default = "default_cadence")]
    pub cadence: String,
}

impl Default for DefaultTier {
    fn default() -> DefaultTier {
        DefaultTier {
            cadence: default_cadence(),
        }
    }
}

fn default_cadence() -> String {
    DEFAULT_CADENCE.to_string()
}

/// A named set of a repository's dependencies, with a cadence (41).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TierDeclaration {
    #[serde(default = "default_cadence")]
    pub cadence: String,
    #[serde(default)]
    pub identities: Vec<String>,
    #[serde(default)]
    pub prefixes: Vec<String>,
    #[serde(default)]
    pub prereleases: bool,
}

/// The profiles named for each seam; a seam left out binds every profile
/// that recognises one of the repository's declarations (51, 67).
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependency_descriptions: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_lookups: Option<Vec<String>>,
}

/// Identities the timber lookup answers for, outright or by a prefix, and
/// the channel it reads for them (C.3, C.7). `component` names the
/// component of the publishing repository where it publishes more than
/// one.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TimberBinding {
    #[serde(default)]
    pub identities: Vec<String>,
    #[serde(default)]
    pub prefixes: Vec<String>,
    pub channel: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
}

impl TimberBinding {
    /// How closely it names `identity`: outright beats a prefix (57).
    fn names(&self, identity: &str) -> Option<Naming> {
        naming(&self.identities, &self.prefixes, identity)
    }
}

/// The file refused, with why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigError(pub String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{PATH}: {}", self.0)
    }
}

impl std::error::Error for ConfigError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Naming {
    Prefix,
    Outright,
}

fn naming(identities: &[String], prefixes: &[String], identity: &str) -> Option<Naming> {
    if identities.iter().any(|named| named == identity) {
        Some(Naming::Outright)
    } else if prefixes
        .iter()
        .any(|prefix| identity.starts_with(prefix.as_str()))
    {
        Some(Naming::Prefix)
    } else {
        None
    }
}

/// A dependency's tier, or the tiers that name it alike (13, 57).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Placed {
    Tier { name: String, prereleases: bool },
    NamedAlike(Vec<String>),
}

impl Config {
    /// The file's text read, or why it is refused. Every identity it names
    /// must be a Package URL (C.1).
    pub fn parse(text: &str) -> Result<Config, ConfigError> {
        let config: Config =
            toml::from_str(text).map_err(|e| ConfigError(e.message().to_string()))?;
        if config.tiers.contains_key(DEFAULT_TIER) {
            return Err(ConfigError(format!(
                "a tier may not be named {DEFAULT_TIER}: [default] sets its cadence"
            )));
        }
        let named = config
            .tiers
            .values()
            .flat_map(|tier| tier.identities.iter())
            .chain(config.timber.iter().flat_map(|t| t.identities.iter()));
        for identity in named {
            if Identity::parse(identity).is_none() {
                return Err(ConfigError(format!(
                    "{identity} is not a Package URL without a version"
                )));
            }
        }
        Ok(config)
    }

    /// The tier that names `identity`: one naming it outright before one
    /// naming it by a prefix, and the default tier where none names it.
    /// Two that name it alike are both given, and headwater chooses
    /// neither (13, 57).
    pub fn place(&self, identity: &Identity) -> Placed {
        let identity = identity.to_string();
        let named: Vec<(Naming, &String, &TierDeclaration)> = self
            .tiers
            .iter()
            .filter_map(|(name, tier)| {
                naming(&tier.identities, &tier.prefixes, &identity).map(|n| (n, name, tier))
            })
            .collect();
        let Some(closest) = named.iter().map(|(n, _, _)| *n).max() else {
            return Placed::Tier {
                name: DEFAULT_TIER.to_string(),
                prereleases: false,
            };
        };
        let alike: Vec<&(Naming, &String, &TierDeclaration)> =
            named.iter().filter(|(n, _, _)| *n == closest).collect();
        match alike.as_slice() {
            [(_, name, tier)] => Placed::Tier {
                name: (*name).clone(),
                prereleases: tier.prereleases,
            },
            several => {
                Placed::NamedAlike(several.iter().map(|(_, name, _)| (*name).clone()).collect())
            }
        }
    }

    /// A tier's cadence, the default tier's included.
    pub fn cadence(&self, tier: &str) -> &str {
        self.tiers
            .get(tier)
            .map_or(self.default.cadence.as_str(), |t| t.cadence.as_str())
    }

    /// The timber binding that answers for `identity`, the closest naming
    /// first (C.3).
    pub fn timber_for(&self, identity: &Identity) -> Option<&TimberBinding> {
        let identity = identity.to_string();
        self.timber
            .iter()
            .filter_map(|binding| binding.names(&identity).map(|n| (n, binding)))
            .max_by_key(|(n, _)| *n)
            .map(|(_, binding)| binding)
    }

    /// Whether the binding names `lookup` for the release lookups, or names
    /// none, which binds every profile (51).
    pub fn binds_release_lookup(&self, lookup: &str) -> bool {
        self.binding
            .release_lookups
            .as_ref()
            .is_none_or(|named| named.iter().any(|n| n == lookup))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = r#"
[default]
cadence = "14d"

[tiers.family]
cadence = "1d"
prefixes = ["pkg:github/boomdrivedev/"]

[tiers.favorites]
identities = ["pkg:github/boomdrivedev/sluice"]
prereleases = true

[tiers.twin]
identities = ["pkg:github/max-sixty/worktrunk"]

[tiers.other-twin]
identities = ["pkg:github/max-sixty/worktrunk"]

[[timber]]
prefixes = ["pkg:github/boomdrivedev/"]
channel = "stable"
"#;

    #[test]
    fn fifty_seven_outright_beats_a_prefix() {
        let config = Config::parse(TEXT).unwrap();
        let place = |id: &str| config.place(&Identity::parse(id).unwrap());
        assert_eq!(
            place("pkg:github/boomdrivedev/sluice"),
            Placed::Tier {
                name: "favorites".into(),
                prereleases: true
            }
        );
        assert_eq!(
            place("pkg:github/boomdrivedev/flume"),
            Placed::Tier {
                name: "family".into(),
                prereleases: false
            }
        );
        assert_eq!(
            place("pkg:github/nixos/nixpkgs"),
            Placed::Tier {
                name: "default".into(),
                prereleases: false
            }
        );
        assert_eq!(config.cadence("default"), "14d");
    }

    #[test]
    fn fifty_seven_two_tiers_alike_are_both_named() {
        let config = Config::parse(TEXT).unwrap();
        assert_eq!(
            config.place(&Identity::github("max-sixty", "worktrunk")),
            Placed::NamedAlike(vec!["other-twin".into(), "twin".into()])
        );
    }

    #[test]
    fn c7_the_timber_binding_names_its_channel() {
        let config = Config::parse(TEXT).unwrap();
        let binding = config
            .timber_for(&Identity::github("boomdrivedev", "crew"))
            .unwrap();
        assert_eq!(binding.channel, "stable");
        assert!(config
            .timber_for(&Identity::github("max-sixty", "worktrunk"))
            .is_none());
        assert!(config.binds_release_lookup("github-releases"));
    }

    #[test]
    fn a_file_that_names_no_package_url_is_refused() {
        assert!(Config::parse("[tiers.x]\nidentities = [\"sluice\"]\n").is_err());
        assert!(Config::parse("[tiers.default]\n").is_err());
        assert!(Config::parse("[nonsense]\n").is_err());
        assert_eq!(Config::parse("").unwrap(), Config::default());
    }
}
