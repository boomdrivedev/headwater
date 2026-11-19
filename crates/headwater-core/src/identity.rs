//! An identity is a Package URL without its version, and a release's
//! identity the same Package URL with its version (2, 10; C.1).

use std::fmt;

use serde::{Serialize, Serializer};

/// A dependency's identity: `pkg:<type>/<namespace>/<name>`, lowercased
/// where its type's names are case-insensitive, as GitHub's are (C.1).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identity {
    kind: String,
    namespace: Option<String>,
    name: String,
}

impl Identity {
    /// `pkg:github/<owner>/<repo>`, both lowercased as Package URL's
    /// github type asks.
    pub fn github(owner: &str, repo: &str) -> Identity {
        Identity {
            kind: "github".to_string(),
            namespace: Some(owner.to_ascii_lowercase()),
            name: repo.to_ascii_lowercase(),
        }
    }

    /// The identity `text` spells, `pkg:<type>/[<namespace>/]<name>`, with
    /// no version, qualifier or subpath; none where it spells none.
    pub fn parse(text: &str) -> Option<Identity> {
        let rest = text.strip_prefix("pkg:")?;
        if rest.contains(['@', '?', '#']) {
            return None;
        }
        let (kind, path) = rest.split_once('/')?;
        if kind.is_empty() || path.is_empty() {
            return None;
        }
        let kind = kind.to_ascii_lowercase();
        let (namespace, name) = match path.rsplit_once('/') {
            Some((namespace, name)) => (Some(namespace.to_string()), name.to_string()),
            None => (None, path.to_string()),
        };
        if name.is_empty() || namespace.as_deref() == Some("") {
            return None;
        }
        if kind == "github" {
            let namespace = namespace?;
            return Some(Identity::github(&namespace, &name));
        }
        Some(Identity {
            kind,
            namespace,
            name,
        })
    }

    /// The Package URL type: `github`, `npm`, `cargo`.
    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// The owner and repository of a `pkg:github` identity.
    pub fn github_repository(&self) -> Option<(&str, &str)> {
        (self.kind == "github").then(|| {
            (
                self.namespace.as_deref().unwrap_or_default(),
                self.name.as_str(),
            )
        })
    }

    /// The release's identity: this Package URL with `version` (10).
    pub fn release(&self, version: &str) -> String {
        format!("{self}@{version}")
    }
}

impl fmt::Display for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.namespace {
            Some(namespace) => write!(f, "pkg:{}/{}/{}", self.kind, namespace, self.name),
            None => write!(f, "pkg:{}/{}", self.kind, self.name),
        }
    }
}

impl Serialize for Identity {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_github_identity_is_lowercased_and_reads_back() {
        let identity = Identity::github("NixOS", "nixpkgs");
        assert_eq!(identity.to_string(), "pkg:github/nixos/nixpkgs");
        assert_eq!(
            Identity::parse("pkg:github/NixOS/nixpkgs"),
            Some(identity.clone())
        );
        assert_eq!(identity.release("1.2.3"), "pkg:github/nixos/nixpkgs@1.2.3");
    }

    #[test]
    fn a_version_or_a_missing_name_is_no_identity() {
        assert_eq!(Identity::parse("pkg:github/jdx/mise@1.0.0"), None);
        assert_eq!(Identity::parse("pkg:github/"), None);
        assert_eq!(Identity::parse("github/jdx/mise"), None);
        assert_eq!(
            Identity::parse("pkg:npm/react").map(|i| i.to_string()),
            Some("pkg:npm/react".to_string())
        );
    }
}
