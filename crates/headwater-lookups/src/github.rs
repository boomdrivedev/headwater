//! The GitHub releases lookup (C.3) and the GitHub release notes lookup
//! (C.4): every release of each `pkg:github` identity's repository, through
//! GraphQL, fifty repositories a query, paged a hundred releases at a time,
//! with the asker's own `gh` sign-in (38). A release's notes are its body,
//! as upstream wrote it (11). GitHub's own flag marks a prerelease, and a
//! draft is no release, since nothing is published yet.

use std::io::Write;
use std::process::{Command, Stdio};

use headwater_core::seams::{Listed, Listing, NotesAnswer, Query, ReleaseLookup};
use headwater_core::Identity;
use serde_json::{json, Value};

/// The profiles' names in a binding and a report.
pub const LOOKUP: &str = "github-releases";
pub const NOTES: &str = "github-release";

/// The most repositories one query asks about (C.3).
pub const BATCH: usize = 50;
/// The releases one page holds.
const PAGE: usize = 100;
/// The most pages read of one repository.
const MOST_PAGES: usize = 30;

/// Whatever answers a GraphQL query: the response's JSON, or why none
/// came.
pub trait GraphQl {
    fn query(&self, query: &str) -> Result<Value, String>;
}

/// GitHub's GraphQL through `gh api graphql`, signed in as the asker.
#[derive(Clone, Debug, Default)]
pub struct GhCli;

impl GraphQl for GhCli {
    fn query(&self, query: &str) -> Result<Value, String> {
        let mut child = Command::new("gh")
            .args(["api", "graphql", "--input", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("gh cannot be started: {e}"))?;
        let body = json!({ "query": query }).to_string();
        child
            .stdin
            .take()
            .expect("stdin is piped")
            .write_all(body.as_bytes())
            .map_err(|e| format!("gh's input cannot be written: {e}"))?;
        let out = child
            .wait_with_output()
            .map_err(|e| format!("gh did not finish: {e}"))?;
        // A query some of whose repositories are missing still answers the
        // rest, and gh exits non-zero: the body says which.
        match serde_json::from_slice::<Value>(&out.stdout) {
            Ok(value) if value.get("data").is_some() => Ok(value),
            _ => Err(format!(
                "gh api graphql failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )),
        }
    }
}

/// The lookup, over any GraphQL endpoint.
pub struct GitHubReleases<G: GraphQl> {
    pub graphql: G,
}

impl<G: GraphQl> GitHubReleases<G> {
    pub fn new(graphql: G) -> Self {
        GitHubReleases { graphql }
    }
}

/// Where one repository's listing stands between pages.
struct Paging<'a> {
    query: &'a Query,
    owner: &'a str,
    name: &'a str,
    after: Option<String>,
    pages: usize,
    releases: Vec<Listed>,
    done: Option<Listing>,
}

impl<G: GraphQl> ReleaseLookup for GitHubReleases<G> {
    fn name(&self) -> &str {
        LOOKUP
    }

    fn list_releases(&self, queries: &[Query]) -> Vec<(Identity, Listing)> {
        let mut paging: Vec<Paging> = queries
            .iter()
            .map(|query| match query.identity.github_repository() {
                Some((owner, name)) => Paging {
                    query,
                    owner,
                    name,
                    after: None,
                    pages: 0,
                    releases: vec![],
                    done: None,
                },
                None => Paging {
                    query,
                    owner: "",
                    name: "",
                    after: None,
                    pages: 0,
                    releases: vec![],
                    done: Some(Listing::Unreached(format!(
                        "{} is no pkg:github identity",
                        query.identity
                    ))),
                },
            })
            .collect();
        loop {
            let open: Vec<usize> = (0..paging.len())
                .filter(|&n| paging[n].done.is_none())
                .collect();
            if open.is_empty() {
                break;
            }
            for batch in open.chunks(BATCH) {
                let text = graphql_query(batch.iter().map(|&n| &paging[n]));
                match self.graphql.query(&text) {
                    Err(reason) => {
                        for &n in batch {
                            paging[n].done = Some(Listing::Unreached(reason.clone()));
                        }
                    }
                    Ok(response) => {
                        for (alias, &n) in batch.iter().enumerate() {
                            read_page(&mut paging[n], &response, alias);
                        }
                    }
                }
            }
        }
        paging
            .into_iter()
            .map(|p| {
                let listing = match p.done {
                    Some(Listing::Listed(_)) | None => Listing::Listed(p.releases),
                    Some(unreached) => unreached,
                };
                (p.query.identity.clone(), listing)
            })
            .collect()
    }
}

fn graphql_query<'a>(paging: impl Iterator<Item = &'a Paging<'a>>) -> String {
    let fields = paging
        .enumerate()
        .map(|(alias, p)| {
            let after = p
                .after
                .as_ref()
                .map(|cursor| format!(", after: {}", Value::from(cursor.as_str())))
                .unwrap_or_default();
            format!(
                "r{alias}: repository(owner: {}, name: {}) {{ releases(first: {PAGE}{after}, orderBy: {{field: CREATED_AT, direction: DESC}}) {{ pageInfo {{ hasNextPage endCursor }} nodes {{ tagName url publishedAt createdAt isPrerelease isDraft description }} }} }}",
                Value::from(p.owner),
                Value::from(p.name)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("query {{\n{fields}\n}}")
}

/// One repository's page from the response, `r<alias>`.
fn read_page(paging: &mut Paging, response: &Value, alias: usize) {
    let key = format!("r{alias}");
    let releases = response
        .get("data")
        .and_then(|d| d.get(&key))
        .and_then(|r| r.get("releases"));
    let Some(releases) = releases.filter(|r| !r.is_null()) else {
        let said = response
            .get("errors")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|e| {
                e.get("path").and_then(|p| p.get(0)).and_then(Value::as_str) == Some(key.as_str())
            })
            .and_then(|e| e.get("message").and_then(Value::as_str))
            .unwrap_or("GitHub gave no releases for it");
        paging.done = Some(Listing::Unreached(format!(
            "{}/{}: {said}",
            paging.owner, paging.name
        )));
        return;
    };
    let scheme = &paging.query.tag_scheme;
    for node in releases
        .get("nodes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let text = |field: &str| node.get(field).and_then(Value::as_str);
        if node.get("isDraft").and_then(Value::as_bool) == Some(true) {
            continue;
        }
        let Some(tag) = text("tagName") else { continue };
        // A tag the scheme does not spell names none of the dependency's
        // versions (43).
        let Some(version) = scheme.version_of(tag) else {
            continue;
        };
        let notes = match text("description") {
            Some(body) if !body.is_empty() => NotesAnswer::Text(body.to_string()),
            _ => NotesAnswer::WroteNothing,
        };
        paging.releases.push(Listed {
            tag: tag.to_string(),
            version,
            date: text("publishedAt")
                .or(text("createdAt"))
                .map(str::to_string),
            link: text("url").unwrap_or_default().to_string(),
            prerelease: node.get("isPrerelease").and_then(Value::as_bool),
            withdrawn: false,
            notes: Some((NOTES.to_string(), notes)),
            upstream_facts: Default::default(),
        });
    }
    paging.pages += 1;
    let info = releases.get("pageInfo");
    let next = info
        .and_then(|i| i.get("hasNextPage"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let cursor = info
        .and_then(|i| i.get("endCursor"))
        .and_then(Value::as_str);
    match (next, cursor) {
        (true, Some(cursor)) if paging.pages < MOST_PAGES => {
            paging.after = Some(cursor.to_string())
        }
        (true, _) => {
            paging.done = Some(Listing::Unreached(format!(
                "{}/{} lists more than {} releases",
                paging.owner,
                paging.name,
                PAGE * MOST_PAGES
            )))
        }
        (false, _) => paging.done = Some(Listing::Listed(vec![])),
    }
}
