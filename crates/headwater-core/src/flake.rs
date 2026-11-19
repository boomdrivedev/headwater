//! The Nix flake dependency description (C.2): each input of the root of
//! `flake.lock`, never one that follows another's, labelled by the input's
//! name, its claim the input's URL in `flake.nix`. The pin is the version
//! the ref the lock records for the input spells as a tag; unknown where
//! that ref spells no version, as a branch does, or the input names a
//! revision or no ref. It reads the lock alone and evaluates no Nix.

use serde_json::Value;

use crate::declaration::{Declaration, Dependency, Deps, Pin, Unread};
use crate::identity::Identity;
use crate::version::TagScheme;

/// The profile's name in a binding and a report.
pub const PROFILE: &str = "nix-flake";

const LOCK: &str = "flake.lock";
const NIX: &str = "flake.nix";

/// What the profile reads of a repository state from its `flake.lock`
/// and `flake.nix`, as they stand; none where it holds no lock (51).
pub fn read(lock: Option<&str>, nix: Option<&str>) -> Option<Deps> {
    let lock = lock?;
    let parsed: Value = match serde_json::from_str(lock) {
        Ok(value) => value,
        Err(error) => return Some(unparsed(format!("the lock does not parse: {error}"))),
    };
    let root_name = parsed.get("root").and_then(Value::as_str).unwrap_or("root");
    let Some(nodes) = parsed.get("nodes").and_then(Value::as_object) else {
        return Some(unparsed("the lock holds no nodes".to_string()));
    };
    let Some(inputs) = nodes
        .get(root_name)
        .and_then(|root| root.get("inputs"))
        .and_then(Value::as_object)
    else {
        return Some(unparsed(format!(
            "the lock's root node {root_name} holds no inputs"
        )));
    };
    let mut deps = Deps::default();
    for (input, target) in inputs {
        // An input that follows another's names a path, a list: it is the
        // other's, and never a declaration of its own (C.2).
        let Some(node_name) = target.as_str() else {
            continue;
        };
        let location = format!("{LOCK}#/nodes/{node_name}");
        let Some(node) = nodes.get(node_name) else {
            deps.unread.push(unread(
                &location,
                format!("the root's input {input} names no node {node_name}"),
            ));
            continue;
        };
        match declaration(input, node, &location, nix) {
            Ok((identity, declaration)) => deps.dependencies.push(Dependency {
                identity,
                declarations: vec![declaration],
            }),
            Err(reason) => deps.unread.push(unread(&location, reason)),
        }
    }
    Some(Deps::merge([deps]))
}

fn unparsed(reason: String) -> Deps {
    Deps {
        dependencies: vec![],
        unread: vec![unread(LOCK, reason)],
    }
}

fn unread(location: &str, reason: String) -> Unread {
    Unread {
        description: PROFILE.to_string(),
        location: location.to_string(),
        reason,
    }
}

fn text<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(Value::as_str)
}

fn declaration(
    input: &str,
    node: &Value,
    location: &str,
    nix: Option<&str>,
) -> Result<(Identity, Declaration), String> {
    let original = node
        .get("original")
        .ok_or_else(|| format!("the input {input} records no original reference"))?;
    let kind = text(original, "type").unwrap_or("unknown");
    if kind != "github" {
        return Err(format!(
            "the input {input} is of type {kind}, and only an input the lock records on GitHub has an identity (C.2)"
        ));
    }
    let (Some(owner), Some(repo)) = (text(original, "owner"), text(original, "repo")) else {
        return Err(format!("the input {input} names no owner and repository"));
    };
    let identity = Identity::github(owner, repo);
    let reference = text(original, "ref");
    let revision = text(original, "rev");
    let says = match reference.or(revision) {
        Some(at) => format!("github:{owner}/{repo}/{at}"),
        None => format!("github:{owner}/{repo}"),
    };
    let (pin, tag_scheme) = match (reference, revision) {
        (Some(reference), _) => match TagScheme::of_ref(reference) {
            Some((scheme, version)) => (Pin::Known(version), Some(scheme)),
            None => (
                Pin::Unknown(format!("the ref {reference} spells no version")),
                None,
            ),
        },
        (None, Some(revision)) => (
            Pin::Unknown(format!(
                "the input names the revision {revision}, not a tag"
            )),
            None,
        ),
        (None, None) => (Pin::Unknown("the input names no ref".to_string()), None),
    };
    Ok((
        identity,
        Declaration {
            description: PROFILE.to_string(),
            location: location.to_string(),
            says,
            pin,
            label: Some(input.to_string()),
            tag_scheme,
            range: None,
            claims: claims(nix, owner, repo),
            upstream: Some(format!("https://github.com/{owner}/{repo}")),
        },
    ))
}

/// The lines of `flake.nix` that give the input's URL: each a place a
/// move of the input changes (vocabulary: *claims*; C.2).
fn claims(nix: Option<&str>, owner: &str, repo: &str) -> Vec<String> {
    let Some(nix) = nix else {
        return vec![];
    };
    let needle = format!("github:{owner}/{repo}").to_ascii_lowercase();
    nix.lines()
        .enumerate()
        .filter(|(_, line)| {
            let line = line.to_ascii_lowercase();
            line.match_indices(&needle).any(|(at, _)| {
                // The repository's name ends there: `/`, a quote, or the line's end.
                matches!(
                    line[at + needle.len()..].chars().next(),
                    None | Some('/' | '"' | '?')
                )
            })
        })
        .map(|(n, _)| format!("{NIX}:{}", n + 1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCK_TEXT: &str = r#"{
      "nodes": {
        "sluice": {"locked": {"type": "github", "owner": "boomdrivedev", "repo": "sluice", "rev": "f16039c"},
                   "original": {"type": "github", "owner": "boomdrivedev", "repo": "sluice", "ref": "v0.97.2"}},
        "nixpkgs": {"original": {"type": "github", "owner": "NixOS", "repo": "nixpkgs", "ref": "nixos-unstable"}},
        "pinned": {"original": {"type": "github", "owner": "plannotator", "repo": "herdr-annotate", "rev": "a9973b4"}},
        "rex": {"original": {"type": "tarball", "url": "https://example.invalid/rex.tar.gz"}},
        "nixpkgs_2": {"original": {"type": "github", "owner": "NixOS", "repo": "nixpkgs"}},
        "root": {"inputs": {"sluice": "sluice", "nixpkgs": "nixpkgs", "pinned": "pinned", "rex": "rex",
                            "followed": ["sluice", "nixpkgs"]}}
      },
      "root": "root",
      "version": 7
    }"#;

    const NIX_TEXT: &str = "{\n  inputs = {\n    sluice.url = \"github:boomdrivedev/sluice/v0.97.2\";\n    sluice-extra.url = \"github:boomdrivedev/sluice-extra\";\n  };\n}\n";

    fn read_fixture() -> Deps {
        read(Some(LOCK_TEXT), Some(NIX_TEXT)).unwrap()
    }

    #[test]
    fn c2_a_tag_ref_is_the_pin_and_its_prefix_the_scheme() {
        let deps = read_fixture();
        let sluice = deps
            .dependencies
            .iter()
            .find(|d| d.identity.to_string() == "pkg:github/boomdrivedev/sluice")
            .unwrap();
        let declaration = &sluice.declarations[0];
        assert_eq!(declaration.pin, Pin::Known("0.97.2".into()));
        assert_eq!(declaration.label.as_deref(), Some("sluice"));
        assert_eq!(declaration.tag_scheme.as_ref().unwrap().prefix, "v");
        assert_eq!(declaration.claims, vec!["flake.nix:3".to_string()]);
        assert_eq!(declaration.location, "flake.lock#/nodes/sluice");
    }

    #[test]
    fn three_a_branch_or_a_revision_is_an_unknown_pin() {
        let deps = read_fixture();
        let pins: Vec<(String, Pin)> = deps
            .dependencies
            .iter()
            .map(|d| (d.identity.to_string(), d.pin()))
            .collect();
        assert!(pins.iter().any(|(id, pin)| id == "pkg:github/nixos/nixpkgs"
            && matches!(pin, Pin::Unknown(w) if w.contains("nixos-unstable"))));
        assert!(pins
            .iter()
            .any(|(id, pin)| id == "pkg:github/plannotator/herdr-annotate"
                && matches!(pin, Pin::Unknown(w) if w.contains("revision"))));
    }

    #[test]
    fn one_another_type_is_unread_naming_its_type_and_a_follow_is_none() {
        let deps = read_fixture();
        assert_eq!(deps.dependencies.len(), 3);
        assert_eq!(deps.unread.len(), 1);
        assert!(deps.unread[0].reason.contains("type tarball"));
    }

    #[test]
    fn one_a_lock_that_does_not_parse_is_unread() {
        let deps = read(Some("{ not json"), None).unwrap();
        assert!(deps.dependencies.is_empty());
        assert_eq!(deps.unread[0].location, "flake.lock");
        assert!(read(None, Some(NIX_TEXT)).is_none());
    }
}
