//! What a dependency description reads of a repository state: every
//! dependency, each with every declaration of it, and every declaration
//! that could not be read, with the reason (1, 2, 4, 40, 53).

use std::collections::BTreeMap;

use serde::Serialize;

use crate::identity::Identity;
use crate::version::{order, Order, TagScheme};

/// The version a declaration makes the repository run, or unknown with why
/// (3; vocabulary: *pin*).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Pin {
    Known(String),
    Unknown(String),
}

impl Pin {
    pub fn version(&self) -> Option<&str> {
        match self {
            Pin::Known(version) => Some(version),
            Pin::Unknown(_) => None,
        }
    }
}

/// One place in a repository state that names a dependency, and what it
/// says of the version (vocabulary: *declaration*; 40).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Declaration {
    /// The dependency description that read it.
    pub description: String,
    /// Where it stands: a path and a pointer into the file.
    pub location: String,
    /// What it says of the version, as written.
    pub says: String,
    pub pin: Pin,
    /// The name it gives its dependency for people and recorders (53).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// How the dependency's tags spell its versions, where it states one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_scheme: Option<TagScheme>,
    /// The versions it accepts, as a Semantic Versioning requirement, where
    /// it accepts more than one (56).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
    /// The other places a move of its dependency changes (vocabulary:
    /// *claims*).
    pub claims: Vec<String>,
    /// Where upstream keeps the dependency's releases and notes, where it
    /// states it (40, 68).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream: Option<String>,
}

/// A declaration that could not be read, with the reason; never dropped
/// from a report (1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Unread {
    pub description: String,
    pub location: String,
    pub reason: String,
}

/// Every declaration of one identity, wherever it sits (2, 4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Dependency {
    pub identity: Identity,
    pub declarations: Vec<Declaration>,
}

impl Dependency {
    /// The dependency's pin: the oldest of its declarations' pins, unknown
    /// when any of them is unknown or two of them cannot be ordered
    /// (vocabulary: *pin*; 4).
    pub fn pin(&self) -> Pin {
        let mut oldest: Option<&str> = None;
        for declaration in &self.declarations {
            let version = match &declaration.pin {
                Pin::Known(version) => version.as_str(),
                Pin::Unknown(why) => {
                    return Pin::Unknown(format!(
                        "{} gives no single version: {why}",
                        declaration.location
                    ))
                }
            };
            oldest = match oldest {
                None => Some(version),
                Some(held) => match order(version, held) {
                    Order::Older => Some(version),
                    Order::Newer | Order::Same => Some(held),
                    Order::Undecided => {
                        return Pin::Unknown(format!(
                            "the declared pins {held} and {version} cannot be ordered"
                        ))
                    }
                },
            };
        }
        oldest.map_or_else(
            || Pin::Unknown("no declaration".to_string()),
            |version| Pin::Known(version.to_string()),
        )
    }

    /// The labels its declarations give, each once, in their order (53).
    pub fn labels(&self) -> Vec<String> {
        let mut labels: Vec<String> = Vec::new();
        for label in self.declarations.iter().filter_map(|d| d.label.clone()) {
            if !labels.contains(&label) {
                labels.push(label);
            }
        }
        labels
    }

    /// The tag scheme its declarations state: the first that states one,
    /// or Semantic Versioning's `v` (C.1).
    pub fn tag_scheme(&self) -> TagScheme {
        self.declarations
            .iter()
            .find_map(|d| d.tag_scheme.clone())
            .unwrap_or_default()
    }
}

/// What the bound dependency descriptions read of one repository state.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Deps {
    pub dependencies: Vec<Dependency>,
    pub unread: Vec<Unread>,
}

impl Deps {
    /// Declarations of one identity become one dependency, wherever they
    /// sit and whichever description read them (2), in identity order.
    pub fn merge(parts: impl IntoIterator<Item = Deps>) -> Deps {
        let mut by_identity: BTreeMap<Identity, Vec<Declaration>> = BTreeMap::new();
        let mut unread = Vec::new();
        for part in parts {
            for dependency in part.dependencies {
                by_identity
                    .entry(dependency.identity)
                    .or_default()
                    .extend(dependency.declarations);
            }
            unread.extend(part.unread);
        }
        Deps {
            dependencies: by_identity
                .into_iter()
                .map(|(identity, declarations)| Dependency {
                    identity,
                    declarations,
                })
                .collect(),
            unread,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declared(location: &str, pin: Pin) -> Declaration {
        Declaration {
            description: "test".into(),
            location: location.into(),
            says: String::new(),
            pin,
            label: None,
            tag_scheme: None,
            range: None,
            claims: vec![],
            upstream: None,
        }
    }

    fn dependency(pins: Vec<Pin>) -> Dependency {
        Dependency {
            identity: Identity::github("o", "r"),
            declarations: pins
                .into_iter()
                .enumerate()
                .map(|(n, pin)| declared(&format!("file{n}"), pin))
                .collect(),
        }
    }

    #[test]
    fn four_the_pin_is_the_oldest_declared() {
        let pin = dependency(vec![Pin::Known("3.2.0".into()), Pin::Known("3.1.0".into())]).pin();
        assert_eq!(pin, Pin::Known("3.1.0".into()));
    }

    #[test]
    fn three_one_unknown_declaration_makes_the_pin_unknown() {
        let pin = dependency(vec![
            Pin::Known("3.2.0".into()),
            Pin::Unknown("a branch".into()),
        ])
        .pin();
        assert!(matches!(pin, Pin::Unknown(why) if why.contains("a branch")));
    }

    #[test]
    fn two_pins_that_cannot_be_ordered_make_it_unknown() {
        let pin = dependency(vec![
            Pin::Known("2026.10".into()),
            Pin::Known("2026.9".into()),
        ])
        .pin();
        assert!(matches!(pin, Pin::Unknown(why) if why.contains("cannot be ordered")));
    }

    #[test]
    fn two_declarations_of_one_identity_are_one_dependency() {
        let part = |location: &str| Deps {
            dependencies: vec![Dependency {
                identity: Identity::github("O", "R"),
                declarations: vec![declared(location, Pin::Known("1.0.0".into()))],
            }],
            unread: vec![],
        };
        let merged = Deps::merge([part("a"), part("b")]);
        assert_eq!(merged.dependencies.len(), 1);
        assert_eq!(merged.dependencies[0].declarations.len(), 2);
    }
}
