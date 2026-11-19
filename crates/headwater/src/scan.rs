//! A scan (model section 5): one reading of the repository state, each
//! dependency asked of the lookup the binding gives it, and the report
//! derived from their answers. Every tier is scanned whether due or not:
//! this version keeps no held state, so every scan reaches its lookups.

use std::collections::BTreeMap;
use std::path::Path;

use headwater_core::config::Config;
use headwater_core::report::Answered;
use headwater_core::seams::{Query, ReleaseLookup};
use headwater_core::{derive, read_repository, Identity, Options, Reading, Report};
use headwater_lookups::timber::Bound;
use headwater_lookups::{github, timber, GhCli, GitHubReleases, HostServer, Timber};

/// The lookups a scan asks: any GitHub GraphQL endpoint and any timber
/// status source, real or recorded.
pub struct Lookups<G: github::GraphQl, S: timber::StatusSource> {
    pub github: G,
    pub timber: S,
}

impl Default for Lookups<GhCli, HostServer> {
    fn default() -> Self {
        Lookups {
            github: GhCli,
            timber: HostServer::default(),
        }
    }
}

/// Which lookup answers for `identity` under the binding (51; C.3): the
/// timber lookup where the binding names it, and then no other; GitHub
/// releases for every other `pkg:github` identity; none otherwise.
fn lookup_for(config: &Config, identity: &Identity) -> Option<&'static str> {
    if config.timber_for(identity).is_some() && config.binds_release_lookup(timber::LOOKUP) {
        return Some(timber::LOOKUP);
    }
    if identity.kind() == "github" && config.binds_release_lookup(github::LOOKUP) {
        return Some(github::LOOKUP);
    }
    None
}

/// The report of the checkout at `repo`.
pub fn scan<G: github::GraphQl, S: timber::StatusSource>(
    repo: &Path,
    lookups: Lookups<G, S>,
    options: &Options,
) -> Report {
    let reading = read_repository(repo);
    let answers = ask(&reading, lookups);
    derive(&reading, &answers, options)
}

/// Every dependency of the reading asked of its lookup, batched by lookup.
pub fn ask<G: github::GraphQl, S: timber::StatusSource>(
    reading: &Reading,
    lookups: Lookups<G, S>,
) -> BTreeMap<Identity, Vec<Answered>> {
    let mut by_lookup: BTreeMap<&str, Vec<Query>> = BTreeMap::new();
    for dependency in &reading.deps.dependencies {
        if let Some(lookup) = lookup_for(&reading.config, &dependency.identity) {
            by_lookup.entry(lookup).or_default().push(Query {
                identity: dependency.identity.clone(),
                tag_scheme: dependency.tag_scheme(),
            });
        }
    }
    let mut answers: BTreeMap<Identity, Vec<Answered>> = BTreeMap::new();
    let mut record = |lookup: &dyn ReleaseLookup, notes: &str, queries: &[Query]| {
        for (identity, listing) in lookup.list_releases(queries) {
            answers.entry(identity).or_default().push(Answered {
                lookup: lookup.name().to_string(),
                notes_lookup: Some(notes.to_string()),
                listing,
            });
        }
    };
    if let Some(queries) = by_lookup.get(github::LOOKUP) {
        record(&GitHubReleases::new(lookups.github), github::NOTES, queries);
    }
    if let Some(queries) = by_lookup.get(timber::LOOKUP) {
        let bound = queries
            .iter()
            .filter_map(|q| {
                reading.config.timber_for(&q.identity).map(|b| Bound {
                    identity: q.identity.clone(),
                    channel: b.channel.clone(),
                    component: b.component.clone(),
                })
            })
            .collect();
        let lookup = Timber {
            source: lookups.timber,
            bound,
        };
        record(&lookup, timber::NOTES, queries);
    }
    answers
}
