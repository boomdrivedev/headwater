//! The lookups against their recorded answers (37): what each sends, and
//! what it lists from what it was given.

use std::cell::RefCell;
use std::collections::VecDeque;

use headwater_core::seams::{Listing, NotesAnswer, Query, ReleaseLookup};
use headwater_core::{Identity, TagScheme};
use headwater_lookups::github::{GitHubReleases, GraphQl};
use headwater_lookups::timber::{Bound, StatusSource, Timber};
use serde_json::Value;

fn answer(name: &str) -> Value {
    let path = format!("{}/tests/answers/{name}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// GraphQL that answers from recorded pages, keeping every query it was
/// sent.
struct Recorded {
    pages: RefCell<VecDeque<Result<Value, String>>>,
    sent: RefCell<Vec<String>>,
}

impl GraphQl for &Recorded {
    fn query(&self, query: &str) -> Result<Value, String> {
        self.sent.borrow_mut().push(query.to_string());
        self.pages
            .borrow_mut()
            .pop_front()
            .expect("no more recorded pages")
    }
}

fn query(owner: &str, repo: &str) -> Query {
    Query {
        identity: Identity::github(owner, repo),
        tag_scheme: TagScheme::default(),
    }
}

/// C.3, 43, 37: GitHub's releases, paged; a draft is none; a tag the
/// scheme does not spell is none; GitHub's flag marks a prerelease; a body
/// is the notes whole; a missing repository is unreached with GitHub's
/// words; and the queries name only the repositories and cursors.
#[test]
fn c3_github_lists_every_release_through_its_pages() {
    let recorded = Recorded {
        pages: RefCell::new(VecDeque::from([
            Ok(answer("github-page-1.json")),
            Ok(answer("github-page-2.json")),
        ])),
        sent: RefCell::new(vec![]),
    };
    let lookup = GitHubReleases::new(&recorded);
    let listed = lookup.list_releases(&[query("boomdrivedev", "sluice"), query("acme", "gone")]);

    let Listing::Listed(sluice) = &listed[0].1 else {
        panic!("sluice was not listed: {:?}", listed[0].1)
    };
    let versions: Vec<(&str, Option<bool>)> = sluice
        .iter()
        .map(|r| (r.version.as_str(), r.prerelease))
        .collect();
    assert_eq!(
        versions,
        vec![("0.97.6", Some(false)), ("0.97.5-rc.1", Some(true))]
    );
    assert_eq!(
        sluice[0].notes,
        Some((
            "github-release".to_string(),
            NotesAnswer::Text("## v0.97.6 (2026-11-19)\r\n\r\n### Fixes\r\n\r\n- the key check reads a symlink as moon does\r\n".into())
        ))
    );
    assert_eq!(sluice[0].date.as_deref(), Some("2026-11-19T10:12:00Z"));
    assert_eq!(
        sluice[1].notes.as_ref().unwrap().1,
        NotesAnswer::WroteNothing
    );
    assert!(matches!(&listed[1].1, Listing::Unreached(why) if why.contains("Could not resolve")));

    let sent = recorded.sent.borrow();
    assert_eq!(sent.len(), 2);
    assert!(sent[0].contains("repository(owner: \"boomdrivedev\", name: \"sluice\")"));
    assert!(sent[0].contains("repository(owner: \"acme\", name: \"gone\")"));
    assert!(sent[1].contains("after: \"Y3Vyc29yOjI=\""));
    assert!(!sent[1].contains("acme"));
}

/// 43: a lookup that cannot be reached says so for every identity asked.
#[test]
fn forty_three_github_unreached() {
    let recorded = Recorded {
        pages: RefCell::new(VecDeque::from([Err(
            "gh api graphql failed: HTTP 502".to_string()
        )])),
        sent: RefCell::new(vec![]),
    };
    let listed = GitHubReleases::new(&recorded).list_releases(&[query("a", "b"), query("c", "d")]);
    assert!(listed
        .iter()
        .all(|(_, l)| matches!(l, Listing::Unreached(why) if why.contains("HTTP 502"))));
}

/// 37, C.3: fifty repositories a query.
#[test]
fn c3_fifty_repositories_a_query() {
    let empty = |n: usize| {
        let data: serde_json::Map<String, Value> = (0..n)
            .map(|i| {
                (
                    format!("r{i}"),
                    serde_json::json!({"releases": {"pageInfo": {"hasNextPage": false, "endCursor": null}, "nodes": []}}),
                )
            })
            .collect();
        Ok(serde_json::json!({ "data": data }))
    };
    let recorded = Recorded {
        pages: RefCell::new(VecDeque::from([empty(50), empty(20)])),
        sent: RefCell::new(vec![]),
    };
    let queries: Vec<Query> = (0..70).map(|n| query("o", &format!("r{n}"))).collect();
    let listed = GitHubReleases::new(&recorded).list_releases(&queries);
    assert_eq!(recorded.sent.borrow().len(), 2);
    assert!(listed.iter().all(|(_, l)| *l == Listing::Listed(vec![])));
}

struct Status(Result<Value, String>);

impl StatusSource for Status {
    fn status(&self, _owner: &str, _name: &str) -> Result<Value, String> {
        self.0.clone()
    }
}

fn bound(channel: &str) -> Bound {
    Bound {
        identity: Identity::github("boomdrivedev", "sluice"),
        channel: channel.into(),
        component: None,
    }
}

/// C.3, C.4, S17: timber's lookup lists the channel's head alone, dated by
/// its promotion, linked to its tag, with the notes its index carries.
#[test]
fn s17_timber_lists_the_channels_head_alone() {
    let lookup = Timber {
        source: Status(Ok(answer("timber-status.json"))),
        bound: vec![bound("stable")],
    };
    let listed = lookup.list_releases(&[query("boomdrivedev", "sluice")]);
    let Listing::Listed(head) = &listed[0].1 else {
        panic!("not listed: {:?}", listed[0].1)
    };
    assert_eq!(head.len(), 1);
    assert_eq!(head[0].version, "0.22.0");
    assert_eq!(head[0].date.as_deref(), Some("2026-11-18T11:06:40Z"));
    assert_eq!(
        head[0].link,
        "https://github.com/boomdrivedev/sluice/releases/tag/v0.22.0"
    );
    assert_eq!(
        head[0].notes.as_ref().unwrap().1,
        NotesAnswer::Text("## v0.22.0\n\n### Features\n\n- the channel moves\n".into())
    );

    let empty = Timber {
        source: Status(Ok(answer("timber-status.json"))),
        bound: vec![bound("beta")],
    };
    assert_eq!(
        empty.list_releases(&[query("boomdrivedev", "sluice")])[0].1,
        Listing::Listed(vec![])
    );

    let missing = Timber {
        source: Status(Ok(answer("timber-status.json"))),
        bound: vec![bound("nightly")],
    };
    assert!(
        matches!(&missing.list_releases(&[query("boomdrivedev", "sluice")])[0].1,
        Listing::Unreached(why) if why.contains("no channel nightly"))
    );
}

/// 43: no serving process, or a refusal, is unreached with the reason.
#[test]
fn forty_three_timber_unreached() {
    let down = Timber {
        source: Status(Err("connection refused".into())),
        bound: vec![bound("stable")],
    };
    assert!(
        matches!(&down.list_releases(&[query("boomdrivedev", "sluice")])[0].1,
        Listing::Unreached(why) if why.contains("connection refused"))
    );
    let refused = serde_json::json!({"answer": "ok", "value": {"channels": {"answer": "refused",
        "refused": {"why": {"not_landed": {"branch": "main"}}}}}});
    let lookup = Timber {
        source: Status(Ok(refused)),
        bound: vec![bound("stable")],
    };
    assert!(
        matches!(&lookup.list_releases(&[query("boomdrivedev", "sluice")])[0].1,
        Listing::Unreached(why) if why.contains("not_landed"))
    );
}
