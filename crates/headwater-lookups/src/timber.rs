//! The timber release lookup (C.3, C.7): for each identity the binding
//! names for it, the head of the channel the binding names, the one
//! accepted version it names (timber 31, 40), as timber answers on the
//! asker's host. The host's one timber serving process answers each
//! repository's status at `/<owner>/<name>/status.json` (timber 132),
//! read from that repository's ledger with the asker's own sign-in; the
//! release is dated by the promotion that moved the channel to it and
//! linked to the version's tag, and its notes are the ones its index
//! carries (timber 134; C.4).
//!
//! Where the binding names the timber lookup for an identity, no other
//! release lookup answers for it (C.3).

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use headwater_core::seams::{Listed, Listing, NotesAnswer, Query, ReleaseLookup};
use headwater_core::Identity;
use serde_json::Value;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// The profiles' names in a binding and a report.
pub const LOOKUP: &str = "timber";
pub const NOTES: &str = "timber";

/// The core's address, where the host's timber serving process answers
/// (timber's `SERVING_HOST` and `SERVING_PORT`).
pub const ADDRESS: &str = "127.0.0.1:7420";

/// Whatever answers a repository's status, as timber's serving process
/// does: the JSON body, or why none came.
pub trait StatusSource {
    fn status(&self, owner: &str, name: &str) -> Result<Value, String>;
}

/// The host's timber serving process, over HTTP/1.0.
#[derive(Clone, Debug)]
pub struct HostServer {
    pub address: SocketAddr,
}

impl Default for HostServer {
    fn default() -> HostServer {
        let address = std::env::var("HEADWATER_TIMBER_ADDRESS")
            .ok()
            .and_then(|a| a.parse().ok())
            .unwrap_or_else(|| ADDRESS.parse().expect("the core's address is an address"));
        HostServer { address }
    }
}

impl StatusSource for HostServer {
    fn status(&self, owner: &str, name: &str) -> Result<Value, String> {
        let unreached = |why: String| {
            format!(
                "timber's serving process at {} did not answer: {why}; `timber status --page` in a checkout of {owner}/{name} starts it",
                self.address
            )
        };
        let mut stream = TcpStream::connect_timeout(&self.address, Duration::from_secs(3))
            .map_err(|e| unreached(e.to_string()))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(60)))
            .map_err(|e| unreached(e.to_string()))?;
        let request = format!(
            "GET /{owner}/{name}/status.json HTTP/1.0\r\nHost: {}\r\nAccept: application/json\r\n\r\n",
            self.address
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|e| unreached(e.to_string()))?;
        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .map_err(|e| unreached(e.to_string()))?;
        let text = String::from_utf8_lossy(&response);
        let body = text.split_once("\r\n\r\n").map_or("", |(_, body)| body);
        serde_json::from_str(body).map_err(|e| unreached(format!("its answer is not JSON: {e}")))
    }
}

/// One identity's binding: the channel read, and the component where the
/// publishing repository names one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bound {
    pub identity: Identity,
    pub channel: String,
    pub component: Option<String>,
}

/// The lookup over a status source, for the identities bound to it.
pub struct Timber<S: StatusSource> {
    pub source: S,
    pub bound: Vec<Bound>,
}

impl<S: StatusSource> ReleaseLookup for Timber<S> {
    fn name(&self) -> &str {
        LOOKUP
    }

    fn list_releases(&self, queries: &[Query]) -> Vec<(Identity, Listing)> {
        queries
            .iter()
            .map(|query| {
                let listing = match self.bound.iter().find(|b| b.identity == query.identity) {
                    None => Listing::Unreached(format!(
                        "the binding names no timber channel for {}",
                        query.identity
                    )),
                    Some(bound) => self.head(query, bound),
                };
                (query.identity.clone(), listing)
            })
            .collect()
    }
}

impl<S: StatusSource> Timber<S> {
    fn head(&self, query: &Query, bound: &Bound) -> Listing {
        let Some((owner, name)) = query.identity.github_repository() else {
            return Listing::Unreached(format!(
                "{} names no repository timber publishes from",
                query.identity
            ));
        };
        match self.source.status(owner, name) {
            Err(reason) => Listing::Unreached(reason),
            Ok(status) => read_head(&status, query, bound, owner, name),
        }
    }
}

/// The channel's head from a repository's status (timber 31, 57, 134).
pub fn read_head(status: &Value, query: &Query, bound: &Bound, owner: &str, name: &str) -> Listing {
    let refused = |what: &str, answer: &Value| {
        Listing::Unreached(format!(
            "timber's status of {owner}/{name} refused its {what}: {}",
            answer
                .get("refused")
                .or(answer.get("unreadable"))
                .unwrap_or(answer)
        ))
    };
    if status.get("answer").and_then(Value::as_str) != Some("ok") {
        return refused("overview", status);
    }
    let channels = &status["value"]["channels"];
    if channels.get("answer").and_then(Value::as_str) != Some("ok") {
        return refused("channels", channels);
    }
    let found = channels["value"]["channels"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|c| {
            let declared = &c["channel"];
            let component = declared["component"].as_str();
            declared["name"].as_str() == Some(bound.channel.as_str())
                && match &bound.component {
                    Some(wanted) => component == Some(wanted.as_str()),
                    None => component.is_none() || component == Some(name),
                }
        });
    let Some(channel) = found else {
        return Listing::Unreached(format!(
            "timber's policy for {owner}/{name} declares no channel {}",
            bound.channel
        ));
    };
    let at = &channel["at"];
    if at.get("answer").and_then(Value::as_str) != Some("ok") {
        return refused("channel's chain", at);
    }
    let at = &at["value"];
    match at["state"].as_str() {
        Some("empty") => Listing::Listed(vec![]),
        Some("at") => {
            let head = &at["head"];
            let Some(version) = head["to"]["version"].as_str() else {
                return Listing::Unreached(format!(
                    "timber's head of {} for {owner}/{name} names no version",
                    bound.channel
                ));
            };
            let date = head["at"]
                .as_i64()
                .and_then(|s| OffsetDateTime::from_unix_timestamp(s).ok())
                .and_then(|t| t.format(&Rfc3339).ok());
            let notes = channel["notes"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|logged| logged["version"].as_str() == Some(version))
                .map(|logged| match logged["notes"]["answer"].as_str() {
                    Some("ok") => match logged["notes"]["value"].as_str() {
                        Some("") | None => NotesAnswer::WroteNothing,
                        Some(text) => NotesAnswer::Text(text.to_string()),
                    },
                    _ => NotesAnswer::Unread(format!(
                        "timber could not render them: {}",
                        logged["notes"]
                    )),
                })
                .unwrap_or_else(|| {
                    NotesAnswer::Unread("timber's status carries no notes for it".to_string())
                });
            let tag = query.tag_scheme.tag_of(version);
            Listing::Listed(vec![Listed {
                link: format!("https://github.com/{owner}/{name}/releases/tag/{tag}"),
                tag,
                version: version.to_string(),
                date,
                prerelease: None,
                withdrawn: false,
                notes: Some((NOTES.to_string(), notes)),
                upstream_facts: Default::default(),
            }])
        }
        other => Listing::Unreached(format!(
            "timber's channel {} for {owner}/{name} is in a state headwater does not read: {other:?}",
            bound.channel
        )),
    }
}
