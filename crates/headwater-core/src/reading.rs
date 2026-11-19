//! One reading of a repository state (31): the checkout as it stands,
//! committed or not, named by its commit and whether it holds changes,
//! read by the bound dependency descriptions and the tiers declaration.
//! Reading writes nothing (5): files are read as plain files, and git is
//! asked only with `--no-optional-locks`, so it refreshes no index.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

use crate::config::{self, Config};
use crate::declaration::{Deps, Unread};
use crate::flake;

/// The repository state a report is of (31).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RepositoryState {
    pub path: PathBuf,
    /// The commit checked out, where git names one.
    pub commit: Option<String>,
    /// Whether the checkout holds changes its commit does not.
    pub changes: bool,
}

/// Everything a scan derives from, made once (31).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    pub state: RepositoryState,
    pub deps: Deps,
    pub config: Config,
    /// The dependency descriptions bound: each that recognised one of the
    /// repository's declarations, or those the binding names (51).
    pub descriptions: Vec<String>,
}

/// The reading of the checkout at `path`.
pub fn read_repository(path: &Path) -> Reading {
    let file = |name: &str| std::fs::read_to_string(path.join(name)).ok();
    let state = RepositoryState {
        path: path.to_path_buf(),
        commit: git(path, &["rev-parse", "HEAD"]),
        changes: git(path, &["status", "--porcelain", "--untracked-files=no"])
            .is_some_and(|s| !s.is_empty()),
    };
    let mut unread_config: Vec<Unread> = Vec::new();
    let config = match file(config::PATH).map(|text| Config::parse(&text)) {
        None => Config::default(),
        Some(Ok(config)) => config,
        Some(Err(error)) => {
            unread_config.push(Unread {
                description: "tiers declaration".to_string(),
                location: config::PATH.to_string(),
                reason: format!("{error}; read as no file, the default tier alone"),
            });
            Config::default()
        }
    };
    let binds = |profile: &str| {
        config
            .binding
            .dependency_descriptions
            .as_ref()
            .is_none_or(|named| named.iter().any(|n| n == profile))
    };
    let mut descriptions = Vec::new();
    let mut parts = Vec::new();
    if binds(flake::PROFILE) {
        if let Some(deps) = flake::read(file("flake.lock").as_deref(), file("flake.nix").as_deref())
        {
            descriptions.push(flake::PROFILE.to_string());
            parts.push(deps);
        }
    }
    parts.push(Deps {
        dependencies: vec![],
        unread: unread_config,
    });
    Reading {
        state,
        deps: Deps::merge(parts),
        config,
        descriptions,
    }
}

fn git(path: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}
